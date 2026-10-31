//! Allocation budgets for cell layout, isolated from other test executables.

#[cfg(test)]
#[cfg(feature = "alloc")]
mod tests {
    //! Tests module for the [`allocations`] module.
    //!
    //! [`allocations`]: super

    use core::{
        alloc::{GlobalAlloc, Layout},
        cell::Cell,
        mem,
        num::NonZero,
    };
    use std::{alloc::System, thread::AccessError, thread_local};

    use aldebaran_print::layout::{Cell as TerminalCell, LayoutOptions, SourceText};

    thread_local! {
        /// Whether allocations on this test thread are being measured.
        static ACTIVE: Cell<bool> = const { Cell::new(false) };
        /// Allocation and reallocation calls made during the measured operation.
        static COUNT: Cell<usize> = const { Cell::new(usize::MIN) };
        /// Total sizes requested by measured allocation and reallocation calls.
        static BYTES: Cell<usize> = const { Cell::new(usize::MIN) };
    }

    /// The system allocator with thread-local counting for measured sections.
    struct Counting;

    /// Process allocator; only the active test thread contributes to measurements.
    #[global_allocator]
    static ALLOCATOR: Counting = Counting;

    /// Count calls without allocating or depending on another thread's activity.
    fn record(size: usize) {
        let _: Result<(), AccessError> = ACTIVE.try_with(|active| {
            if active.get() {
                let _: Result<(), AccessError> = COUNT.try_with(|count| count.set(count.get() + 1));
                let _: Result<(), AccessError> = BYTES.try_with(|bytes| bytes.set(bytes.get() + size));
            }
        });
    }

    // SAFETY: Every allocation operation is forwarded unchanged to System;
    // thread-local counters neither modify allocation arguments nor access memory.
    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            record(layout.size());
            // SAFETY: The caller supplies the valid GlobalAlloc layout, forwarded unchanged.
            unsafe { System.alloc(layout) }
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            record(layout.size());
            // SAFETY: The caller supplies the valid GlobalAlloc layout, forwarded unchanged.
            unsafe { System.alloc_zeroed(layout) }
        }

        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            // SAFETY: The caller's allocation and layout are forwarded to their allocator.
            unsafe { System.dealloc(pointer, layout) }
        }

        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            record(size);
            // SAFETY: The caller's allocation, layout, and new size are forwarded unchanged.
            unsafe { System.realloc(pointer, layout, size) }
        }
    }

    /// Allocation traffic during one measured operation.
    #[derive(Debug, PartialEq, Eq)]
    // NOTE(invariant): None.
    struct AllocationUsage {
        /// Allocation and reallocation calls.
        call_count: usize,

        /// Sum of requested allocation sizes, including replacement buffers.
        requested_bytes: usize,
    }

    /// Run an operation with already-initialized thread-local counters.
    fn measure<T, F>(operation: F) -> (T, AllocationUsage)
    where
        F: FnOnce() -> T,
    {
        COUNT.set(usize::MIN);
        BYTES.set(usize::MIN);
        ACTIVE.set(true);

        let result = operation();

        ACTIVE.set(false);

        let usage = AllocationUsage {
            call_count: COUNT.get(),
            requested_bytes: BYTES.get(),
        };

        (result, usage)
    }

    #[test]
    fn borrowed_string_layout_uses_two_row_buffers_independent_of_line_length() {
        let layout_options = LayoutOptions::new(NonZero::new(80).expect("columns"), NonZero::new(4).expect("tabs"));
        let mut baseline = None;

        for length in [160, 8192] {
            let source = "x".repeat(length);

            let ((cell_count, row_count), usage) = measure(|| {
                let mut layout = SourceText::from(source.as_str()).layout(layout_options);
                let mut cell_count = usize::MIN;
                let mut row_count = usize::MIN;

                while let Some(row) = layout.next_row() {
                    cell_count += row.cells().len();
                    row_count += 1;
                }

                (cell_count, row_count)
            });

            assert!(usage.call_count <= 2, "row buffers needed {usage:?}");
            assert!(
                usage.requested_bytes <= 80 * (mem::size_of::<TerminalCell>() + char::MAX.len_utf8()),
                "output storage grew with a {length}-byte source line: {usage:?}"
            );
            assert_eq!(cell_count, length);
            assert_eq!(row_count, length.div_ceil(80));

            if let Some(expected) = &baseline {
                assert_eq!(&usage, expected, "storage grew with source length");
            } else {
                baseline = Some(usage);
            }
        }
    }

    #[test]
    fn reserved_component_input_uses_four_bulk_allocations_without_per_row_growth() {
        let layout_options = LayoutOptions::new(NonZero::new(80).expect("columns"), NonZero::new(4).expect("tabs"));

        for length in [8, 8192] {
            let (cell_count, usage) = measure(|| {
                let mut text = SourceText::with_capacity(10, length);

                for _ in usize::MIN..length {
                    text.push(NonZero::<usize>::MIN, &'x').expect("component visualization");
                }

                let mut layout = text.layout(layout_options);
                let mut cell_count = usize::MIN;

                while let Some(row) = layout.next_row() {
                    if let usize::MIN = cell_count {
                        assert_eq!(row.source_range().start, 10);
                    }

                    cell_count += row.cells().len();
                }

                cell_count
            });

            assert!(usage.call_count <= 4, "{length} components needed {usage:?}");
            assert_eq!(cell_count, length);
        }
    }

    #[test]
    fn unicode_controls_and_tabs_reuse_storage_across_wrapped_rows() {
        for budget in [1, 3, 80] {
            let layout_options = LayoutOptions::new(NonZero::new(budget).expect("columns"), NonZero::new(4).expect("tabs"));
            let mut baseline = None;

            for repetitions in [100, 10_000] {
                let source = "👩🏽‍💻\t\u{301}e\u{301}🇧🇷\r\n".repeat(repetitions);

                let (cell_count, usage) = measure(|| {
                    let mut layout = SourceText::from(source.as_str()).layout(layout_options);
                    let mut cell_count = usize::MIN;

                    while let Some(row) = layout.next_row() {
                        cell_count += row.cells().len();
                    }

                    cell_count
                });

                assert!(cell_count > usize::MIN);

                if let Some(expected) = &baseline {
                    assert_eq!(&usage, expected, "storage grew with line length at width {budget}");
                } else {
                    baseline = Some(usage);
                }
            }
        }
    }

    #[test]
    fn long_combining_clusters_grow_the_row_buffer_once_then_reuse_it() {
        let mut cluster = "\u{301}".repeat(4096);

        cluster.insert(usize::MIN, 'e');

        let layout_options = LayoutOptions::new(NonZero::<usize>::MIN, NonZero::<usize>::MIN);
        let mut baseline = None;

        for repetitions in [2, 64] {
            let source = cluster.repeat(repetitions);

            let (row_count, usage) = measure(|| {
                let mut layout = SourceText::from(source.as_str()).layout(layout_options);
                let mut row_count = usize::MIN;

                while let Some(row) = layout.next_row() {
                    assert_eq!(row.cells().len(), 1);
                    row_count += 1;
                }

                row_count
            });

            assert_eq!(row_count, repetitions);
            assert!(
                usage.requested_bytes <= 4 * cluster.len() + 2 * mem::size_of::<TerminalCell>(),
                "buffer storage grew beyond one combining cluster: {usage:?}"
            );

            if let Some(expected) = &baseline {
                assert_eq!(&usage, expected, "row storage grew with repeated clusters");
            } else {
                baseline = Some(usage);
            }
        }
    }
}
