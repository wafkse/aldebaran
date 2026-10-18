# Aldebaran Report Code Generation

`aldebaran-report-macro` derives diagnostic headings and source annotations
alongside standard error behavior. `report` attributes describe the diagnostic
heading, annotation message, and source target, while separate `error`
attributes specify display and error-source behavior. The diagnostic type must
also provide `Debug`.

Enable `codegen` on `aldebaran-report` and use its `codegen::Report` derive, or
enable `report-codegen` on the `aldebaran` facade and use
`aldebaran::report::codegen::Report`. Generated report paths support renamed
dependencies and prefer the direct runtime dependency when both are present.

Structs can describe an inferred primary location, expose a stored annotation
collection, or forward a selected field's report. Enum alternatives each wrap
exactly one unnamed field and supply custom wording or transparent wording.
Custom reports and all derived enums use `Simple::Error`, while transparent
structs preserve the wrapped report's classification and annotation collection.

The [`Report` derive documentation](https://docs.rs/aldebaran-report-macro/latest/aldebaran_report_macro/derive.Report.html)
describes attribute forms and projection contracts. Source targets retain the
caller's coordinates without source validation, so the report must later be
paired with the source those coordinates describe.

The [facade diagnostics example](https://github.com/wafkse/aldebaran/blob/development/crates/aldebaran/README.md#diagnostics)
shows a diagnostic declaration using the facade's derive path.
