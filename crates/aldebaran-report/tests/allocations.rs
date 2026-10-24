//! Allocation budgets for complete Fancy renders with an allocation-free sink.

#[cfg(test)]
mod tests {
    use aldebaran_baseline::prelude::Span;
    use aldebaran_report::{
        prelude::{Label, oneshot},
        render::{
            RenderMut,
            textual::{
                Present, Textual,
                offload::fancy::{Colored, Fancy, FancySettings, LayoutSettings, charset::Charset},
                viewport::{Extents, Viewport},
            },
        },
    };
    use core::{
        alloc::{GlobalAlloc, Layout},
        cell::Cell,
        fmt,
        num::NonZero,
    };
    use std::{alloc::System, thread::AccessError, thread_local};

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

    /// Counts destination writes without allocating a diagnostic string.
    #[derive(Default)]
    // NOTE(invariant): None.
    struct Sink {
        /// Number of sink calls, including empty writes.
        write_count: usize,
    }

    impl fmt::Write for Sink {
        fn write_str(&mut self, _: &str) -> fmt::Result {
            let Self { write_count } = self;

            *write_count += 1;

            Ok(())
        }
    }

    #[test]
    fn wrapped_lines_keep_a_constant_allocation_budget() {
        let mut baseline = None;

        for length in [160, 4 * 1024, 64 * 1024] {
            let source = "a".repeat(length);
            let report = oneshot::single(source.as_str(), Label::new("detail", Span::ALL));
            let fancy_settings = FancySettings::tuple((Colored::No, LayoutSettings::marginal(), Charset::UNICODE));
            let dimension = NonZero::new(80).expect("viewport width");
            let present = Present::raw(fancy_settings, Viewport::new(Extents::tuple((dimension, dimension))));
            let mut sink = Sink::default();
            let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut sink);

            let (result, usage) = measure(|| renderer.render_mut_with_input(present, &report));

            result.expect("render wrapped source");

            assert!(usage.call_count <= 10, "wrapped rows allocated: {usage:?}");
            assert!(usage.requested_bytes <= 8192, "source-sized metadata allocated: {usage:?}");
            assert!(sink.write_count < length / 4 + 200, "marker output must use bounded chunks");

            if let Some(expected) = &baseline {
                assert_eq!(&usage, expected, "allocation traffic must not grow with wrapped row count");
            }

            baseline = Some(usage);
        }
    }

    #[test]
    fn physical_lines_reuse_string_and_byte_source_storage() {
        for content in ["abcdefgh\n", "中\te\u{301}\n"] {
            let mut string_baseline = None;
            let mut byte_baseline = None;

            for line_count in [10, 100, 1000] {
                let source = content.repeat(line_count);
                let dimension = NonZero::new(80).expect("viewport width");
                let viewport = Viewport::new(Extents::tuple((dimension, dimension)));
                let fancy_settings = FancySettings::tuple((Colored::No, LayoutSettings::marginal(), Charset::UNICODE));
                let report = oneshot::single(source.as_str(), Label::new("detail", Span::ALL));
                let mut sink = Sink::default();
                let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut sink);

                let (result, usage) = measure(|| renderer.render_mut_with_input(Present::raw(fancy_settings, viewport), &report));

                result.expect("render string lines");

                assert!(usage.call_count <= 16, "physical lines allocated: {usage:?}");
                assert!(usage.requested_bytes <= 4096, "collected line metadata: {usage:?}");

                if let Some(expected) = &string_baseline {
                    assert_eq!(&usage, expected, "string scratch must survive line transitions");
                }

                string_baseline = Some(usage);

                let report = oneshot::single(source.as_bytes(), Label::new("detail", Span::ALL));
                let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut sink);

                let (result, usage) = measure(|| renderer.render_mut_with_input(Present::raw(fancy_settings, viewport), &report));

                result.expect("render byte lines");

                assert!(usage.call_count <= 24, "byte visualization buffers were recreated: {usage:?}");
                assert!(usage.requested_bytes <= 4096, "byte scratch grew with line count: {usage:?}");

                if let Some(expected) = &byte_baseline {
                    assert_eq!(&usage, expected, "generic visualization scratch must survive line transitions");
                }

                byte_baseline = Some(usage);
            }
        }
    }
}
