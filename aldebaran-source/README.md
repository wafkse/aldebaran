# Source

The `source` module provides a unified interface for handling various types of source sequences in Rust. This module is designed to facilitate error reporting and manipulation of different source types, such as strings, slices, and custom types.

## Overview

The `Source` trait and its augmentative traits define a comprehensive framework for working with source sequences. A source sequence is a one-dimensional plane of potentially variable-sized components, which can be used as an error source.

### Key Features

- **Unified Interface**: The `Source` trait serves as a base trait for all types that can be used as error sources.
- **Augmentative Traits**: Additional traits like `SourceLines`, `SourceDissect`, `SourceIter`, and `SourceMetadata` extend the functionality of the `Source` trait.
- **Default Terminator**: The `DefaultTerminator` type alias simplifies the determination of the default terminator for a source.
- **Footprint and Size**: Methods to determine the footprint and size of a source sequence.
- **Start and End Spans**: Methods to get the start and end spans of a source sequence.

## Modules

- `component`: Defines the `Component` trait for source components.
- `diff`: Provides the `SourceDiff` trait for differential sources.
- `dissect`: Contains the `SourceDissect` trait for dissecting sources.
- `hash`: Includes the `SourceHash` trait for hashing sources.
- `iter`: Offers the `SourceIter` trait for iterating over sources.
- `line`: Implements the `SourceLines` trait for line-based interfaces.
- `location`: Defines the `Location` trait for source locations.
- `metadata`: Contains the `SourceMetadata` trait for source metadata.
- `owned`: Provides the `SourceOwned` trait for owned sources.
- `terminate`: Defines the `Terminated` trait for terminated sources.

## Example Usage

Here is a basic example of how to implement the `Source` trait for a custom type:

```rust
use source::{Source, Span, Component};

struct MySource<'a> {
    data: &'a str,
}

impl<'a> Source<'a> for MySource<'a> {
    type Component = char;

    const EMPTY_SOURCE: &'a Self = &MySource { data: "" };

    fn footprint(&self) -> Option<Span> {
        if self.data.is_empty() {
            None
        } else {
            Some(Span::new(0, self.data.len()))
        }
    }
}
```

## Conclusion

The `source` module provides a powerful and flexible framework for working with various types of source sequences in Rust. By leveraging the `Source` trait and its augmentative traits, developers can create robust and generic error reporting systems.

For more detailed information, refer to the documentation of each trait and module within the `source` module.