Aldebaran's `text` crate provides text and lexical scanning capabilities in a both performant and developer-friendly way.

# Structure

This crate is composed of the common 3-in-1 structure, `core`, `macro`, and the "glue" crate, which is the one that re-exports everything outside.

However, in this case, there is no `macro` crate due to not requiring procedural macros for the overall `text` crate.
