#![doc = include_str!("../README.md")]

mod report;

use proc_macro::TokenStream;

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

/// The report namespace available to the deriving crate, including renamed dependencies.
#[derive(Debug)]
// NOTE(invariant): None.
struct RuntimePath(
    /// Namespace providing the reporting traits and their supporting re-exports.
    TokenStream2,
);

impl RuntimePath {
    /// Select an accessible report namespace, preferring the direct runtime over the facade.
    fn resolve() -> syn::Result<Self> {
        match crate_name("aldebaran-report") {
            Ok(found) => Ok(Self(Self::direct(found))),
            Err(_) => match crate_name("aldebaran") {
                Ok(found) => Ok(Self(Self::facade(found))),
                Err(error) => Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("failed to resolve Aldebaran report runtime: {error}"),
                )),
            },
        }
    }

    /// The direct runtime namespace under its name in the deriving crate.
    fn direct(found: FoundCrate) -> TokenStream2 {
        match found {
            FoundCrate::Itself => quote!(crate),
            FoundCrate::Name(name) => {
                let ident = format_ident!("{}", name);

                quote!(::#ident)
            }
        }
    }

    /// The facade's report namespace under its name in the deriving crate.
    fn facade(found: FoundCrate) -> TokenStream2 {
        match found {
            FoundCrate::Itself => quote!(crate::report),
            FoundCrate::Name(name) => {
                let ident = format_ident!("{}", name);

                quote!(::#ident::report)
            }
        }
    }
}

/// Derives [`Annotated`](https://docs.rs/aldebaran-report/latest/aldebaran_report/annotated/trait.Annotated.html)
/// and [`Report`](https://docs.rs/aldebaran-report/latest/aldebaran_report/report/trait.Report.html)
/// together with [`Display`](core::fmt::Display) and [`Error`](core::error::Error).
///
/// Separate `report` and `error` attributes control diagnostic content and standard
/// error behavior. Supply an `error` declaration such as `#[error("invalid value")]`
/// on a struct or each enum variant, and provide [`Debug`](core::fmt::Debug)
/// separately. Report headings and annotation messages are independent of error
/// display text. Use separate `#[error(transparent(field))]` or
/// `#[error(transparent(0))]` metadata for standard error forwarding.
///
/// # Struct reports
///
/// A custom struct requires exactly one of `title = "text"` or `title_with = path`
/// and exactly one of `message = "text"` or `message_with = path` inside `#[report(...)]`.
/// Projection functions receive `&Self`, with results coercible to `&str` for
/// that borrow. Text literals are used verbatim, without format interpolation.
///
/// The primary target is inferred from exactly one field whose final type path
/// segment is `Span`. When no field has that spelling, a sole field must implement
/// `Annotated` and supplies the target instead. Aliases are not resolved, and
/// multiple `Span` fields are ambiguous. The struct acts as its own annotation
/// and uses `Simple::Error` as its classification.
///
/// `annotations = field` or `annotations = 0` selects a stored collection implementing
/// `PrimaryAnnotations`. A title is still required, but a message is optional
/// because the enclosing value's `Annotated` view uses the collection's primary
/// message and target by default. A message override changes that view's message
/// to `str`, while `Report::annotations` still exposes the stored collection unchanged.
///
/// `#[report(transparent)]` forwards a sole field, while
/// `#[report(transparent(field))]` or `#[report(transparent(0))]` selects a field
/// explicitly. The field must implement both `Annotated` and `Report`, and cannot
/// have a type path ending in `Span`. This mode preserves the wrapped message,
/// target, heading, classification, and annotation collection and excludes all
/// other report options.
///
/// # Enum reports
///
/// Every variant must wrap exactly one unnamed field. A custom variant requires
/// `title = "text"` and exactly one of `message = "text"` or `message_with = path`.
/// Its message projection receives the borrowed payload, with a result coercible
/// to `&str` for that borrow. A payload type path ending in `Span` supplies the
/// target directly, while other payloads require `Annotated` to supply it.
///
/// `#[report(transparent)]` instead reuses the payload's annotation message and
/// report heading, each coercible to `&str` for the payload borrow, together with
/// its annotation target.
/// Derived enums always expose themselves as their annotation and use `Simple::Error`,
/// so transparent variants do not expose a wrapped annotation collection or classification.
/// Enum variants support neither `title_with`, stored annotations, nor explicit
/// transparent field selectors.
///
/// # Dependencies and type requirements
///
/// Generated report paths use the deriving crate's direct `aldebaran-report`
/// dependency or the `aldebaran::report` facade, including renamed dependencies.
/// Report implementations preserve declared generics and where clauses without
/// inferring additional field bounds, so those bounds must already support the
/// chosen report policy. Struct and variant options must have recognized names
/// and cannot be repeated or conflict, and unions are unsupported.
/// Targets are not checked against any source, and callers
/// must pair reports with a source using the same coordinates.
#[proc_macro_derive(Report, attributes(error, report))]
#[inline]
pub fn report_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = RuntimePath::resolve().and_then(|runtime| {
        let RuntimePath(runtime) = runtime;

        let error = fack_codegen::generate(&input)?;
        let report = report::Report::new(input)?.expand(&runtime);

        Ok(quote! {
            #error
            #report
        })
    });

    match expanded {
        Ok(expanded) => TokenStream::from(expanded),
        Err(error) => TokenStream::from(error.into_compile_error()),
    }
}

#[cfg(test)]
mod tests {
    use super::report::Report;

    fn parse(input: &str) -> syn::Result<Report> {
        Report::new(syn::parse_str(input)?)
    }

    #[test]
    fn accepts_generic_structures_and_enum_alternatives() {
        parse(
            r#"
                #[report(title = "generic", message_with = render_message)]
                struct GenericIssue<T> { target: Span, value: T }
            "#,
        )
        .expect("valid generic report metadata parses");

        parse(
            r#"
                enum Issue {
                    #[report(title = "source", message = "bad source")]
                    Source(Span),
                    #[report(transparent)]
                    Wrapped(InnerIssue),
                }
            "#,
        )
        .expect("valid enum alternatives parse");
    }

    #[test]
    fn rejects_malformed_and_duplicate_darling_metadata() {
        let malformed = parse(
            r#"
                #[report(title = 42, message = "bad input")]
                struct Issue(Span);
            "#,
        );
        assert!(malformed.is_err(), "title metadata must be a string literal");

        let duplicate = parse(
            r#"
                #[report(title = "first", title = "second", message = "bad input")]
                struct Issue(Span);
            "#,
        );
        assert!(duplicate.is_err(), "Darling must reject duplicate declarations");
    }

    #[test]
    fn rejects_conflicting_policies_and_invalid_targets() {
        let conflicting = parse(
            r#"
                #[report(title = "static", title_with = project_title, message = "bad input")]
                struct Issue(Span);
            "#,
        );
        assert!(conflicting.is_err(), "static and projected titles are exclusive");

        let invalid_target = parse(
            r#"
                #[report(title = "stored", annotations = missing)]
                struct Issue { annotations: Annotations }
            "#,
        );
        assert!(invalid_target.is_err(), "selected annotation members must exist");

        let invalid_variant = parse(
            r#"
                enum Issue {
                    #[report(title = "invalid", message = "bad input")]
                    Pair(Span, Span),
                }
            "#,
        );
        assert!(invalid_variant.is_err(), "report alternatives wrap one tuple field");
    }
}
