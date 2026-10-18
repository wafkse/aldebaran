# Aldebaran Source

`aldebaran-source` provides source access for lexers, parsers, and diagnostic
renderers. It supports strings and byte slices, and applications can implement
its traits for other input types.

## Components and Coordinates

A source has components to read and coordinates that locate them. For a string,
the components are Unicode scalar values, while coordinates count UTF-8 bytes.
A character can therefore occupy several coordinates. Byte slices use bytes for
both components and coordinates.

[`Source`] describes this relationship. Separate traits provide iteration,
source views, content comparison and hashing, metadata, and owned conversion,
so an input type can support the operations its consumers need.

## Lines and Annotations

Line views keep content separate from its terminator and retain positions in the
original source. This lets diagnostic renderers display an excerpt using the
same locations as the lexer or parser. The default termination policy recognizes
LF and CRLF, and applications can supply another policy.

Line segmentation associates annotations with views of the selected content.
See the [`SourceLines`] API for line selection and boundary behavior.

## Allocation and Metadata

The crate is `no_std` and uses `alloc` for operations such as collecting line
annotations. Source metadata can provide a printable origin independently of
the input, allowing a diagnostic to identify where that input came from.

[`Source`]: https://docs.rs/aldebaran-source/latest/aldebaran_source/source/trait.Source.html
[`SourceLines`]: https://docs.rs/aldebaran-source/latest/aldebaran_source/line/trait.SourceLines.html
