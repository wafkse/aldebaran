# Aldebaran Print

`aldebaran-print` provides a small sink-oriented printing contract for values that should not commit to `core::fmt::Display`.

The central `Print` trait writes directly into any `core::fmt::Write` sink. A printable value may borrow external context such as an interner or arena, while values with a default context retain the simpler direct `print` path. Printing performs no allocation by itself.

`Print::display` and `Print::display_with_ctx` expose borrowed printable values through `core::fmt::Display` without making the printable type itself commit to `Display`.

The `Combine` extension trait provides deferred sequencing and repetition. These adaptors store their input values and preserve left-to-right formatting errors without materializing an intermediate string.

The crate has no dependencies and is suitable for `no_std` consumers.
