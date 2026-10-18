# Aldebaran ANSI

`aldebaran-ansi` adds terminal color and text styling to values printed through
[`aldebaran-print`](https://docs.rs/aldebaran-print). By applying styling at
printing time, it keeps presentation choices separate from source and
diagnostic values.

`Paintable` builders retain color and attribute choices until output is written.
Color choices replace earlier settings for the same surface, while attributes
accumulate. Conditional painting stores the enable decision when configured,
so later printing does not reevaluate the predicate. Enabled painting ends
output with a global SGR reset, which clears settings without restoring an
enclosing style.

Because the crate writes sequences without detecting terminal support, callers
decide when styling should be enabled. This also lets the formatting code run
without a terminal discovery layer, and the crate is `no_std` outside tests.
