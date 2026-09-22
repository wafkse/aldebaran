# Heap

`aldebaran-heap` provides a `no_std` allocator wrapper for Aldebaran.

It uses the stable `allocator-api2` interface and exposes `Box` and `Vec`
aliases that default to the [`Heap`](crate::heap::Heap) allocator.
