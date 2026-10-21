# Aldebaran Reports

`aldebaran-report` defines diagnostic headings, classifications, and annotations.
A `Report` can retain locations independently of source ownership. Attaching
the source produces a `SourceReport` whose targets use that source's coordinates.
For string sources those coordinates count UTF-8 bytes.

Use `InlineAnnotations` for fixed collections, `Nonempty` for a growable
collection, or `RefNonempty` to borrow a primary annotation and related context.
The required first value identifies the primary location, while related values
retain caller order.

```rust
use aldebaran_baseline::{nonempty::Nonempty, span::Span};
use aldebaran_report::prelude::{Fancy, Label, RenderMut, Textual, oneshot};

let primary_span = Span::unit(4);
let related_span = Span::unit(0);
let primary = Label::new("unused binding", primary_span);
let related = Label::new("in this declaration", related_span);

let mut annotations = Nonempty::single(primary);
annotations.list_mut().push(related);

let report = oneshot::many("let x = 1;", "unused binding", annotations.as_ref());
let mut output = String::new();
let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut output);
renderer.render_mut(&report).expect("render diagnostic");

assert!(output.contains("unused binding"));
```

`Fancy` renders source excerpts, `Hexdump` displays byte sources, and `Summarized`
prints locations and messages without source text. Textual rendering appends to
the sink and can leave partial output after a formatting error. Enable `codegen`
for the `Report` derive, or `report-codegen` through the facade.
