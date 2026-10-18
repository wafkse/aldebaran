# Aldebaran Foundations

`aldebaran-baseline` provides shared primitives for Aldebaran's source
processing and diagnostic crates. These foundations let frontends exchange
coordinate and collection data without coupling lexers, parsers, and renderers
to one another.

The crate is `no_std`, with allocation-dependent collections enabled by the
default `alloc` feature. If only source coordinates and fixed-capacity storage
are needed, that feature can be disabled. The separate `allocator` feature
adds integration with `allocator-api2`.

