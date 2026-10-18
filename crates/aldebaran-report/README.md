# Aldebaran Reports

`aldebaran-report` defines diagnostic headings, classifications, and annotations.
A `Report` can retain locations independently of source ownership. Attaching
the source produces a `SourceReport` whose targets use that source's coordinates.
For string sources those coordinates count UTF-8 bytes.

`Fancy` renders source excerpts, `Hexdump` displays byte sources, and `Summarized`
prints locations and messages without source text. Textual rendering appends to
the sink and can leave partial output after a formatting error. Enable `codegen`
for the `Report` derive, or `report-codegen` through the facade.
