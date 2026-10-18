# Aldebaran Source

`aldebaran-source` defines the source capabilities and coordinates shared by
frontends and diagnostic renderers. `Source` relates component widths to source
spans, while separate traits provide iteration, dissection, content identity,
metadata, and owned conversion. Applications can implement these capabilities
for their input types without requiring every source operation at once.

Built-in string sources yield Unicode scalar values but measure spans in UTF-8
bytes. Byte slices use bytes for both components and coordinates. Line views
retain absolute spans and separate content from terminators, so renderers can
select lines and project annotations without rebasing source locations. The
default line policy recognizes LF and CRLF, and callers can supply another
termination policy.

The crate is `no_std` with allocation support for operations such as collecting
line annotations. Source metadata can provide a printable origin independently
of the content and its coordinates.
