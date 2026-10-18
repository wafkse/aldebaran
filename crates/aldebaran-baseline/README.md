# Aldebaran Foundations

`aldebaran-baseline` provides shared primitives for Aldebaran's source
processing and diagnostic crates. These foundations let frontends exchange
coordinate and collection data without coupling lexers, parsers, and renderers
to one another.

The crate is `no_std`, with allocation-dependent collections enabled by the
default `alloc` feature. If only source coordinates and fixed-capacity storage
are needed, that feature can be disabled. The separate `allocator` feature
adds integration with `allocator-api2`.

`Nonempty` retains a required first value while its tail can grow or shrink.
Use its `RefNonempty` view when an API needs to share the sequence without
cloning elements or allocating another collection.

```rust
use aldebaran_baseline::nonempty::Nonempty;

let mut values = Nonempty::single("primary");
values.list_mut().push("related");

let view = values.as_ref();
assert_eq!(view.first(), &"primary");
assert_eq!(view.rest(), &["related"]);

values.list_mut().clear();
assert_eq!(values.len().get(), 1);
```
