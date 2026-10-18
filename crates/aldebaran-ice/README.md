# Aldebaran Internal Compiler Errors

`aldebaran-ice` lets compiler components signal internal failures with a panic,
while malformed source is reported through frontend diagnostics. Keeping this
internal failure mechanism separate from the diagnostic framework allows
foundational components to use it through a `no_std` dependency.

`Ice` provides a fixed-message failure path and helpers for optional or fallible
operations that a compiler invariant requires to succeed. Callers establish
that assumption before using a helper. Error-bearing failures include debug
information, while absent optional values require no debug implementation.
