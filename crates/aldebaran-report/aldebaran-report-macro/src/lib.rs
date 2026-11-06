#![doc = include_str!("../../../../docs/part/COMPONENT.md")]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../../../../docs/part/FACADE.md")]
#![doc = include_str!("../../../../docs/part/LICENSE.md")]

mod forward;
mod report;

use proc_macro::TokenStream;

use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Runtime path used by generated report implementations.
///
/// Generated code uses the `aldebaran` facade by default and honors Cargo aliases.
/// Add `#[aldebaran(crate)]` to use a direct `aldebaran-report` dependency instead.
#[derive(Debug)]
// NOTE(invariant): Tokens = AccessibleRuntimeNamespace(DeclarationPolicy)
struct RuntimePath(
    /// Tokens naming the report runtime visible from generated code.
    TokenStream2,
);

impl RuntimePath {
    /// Resolve either the direct runtime crate or the public facade.
    fn resolve(attributes: &[syn::Attribute]) -> syn::Result<Self> {
        let policy = attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("aldebaran"))
            .try_fold(None, |policy, attribute| match policy {
                None => attribute.parse_args::<syn::Token![crate]>().map(Some),
                Some(_) => Err(syn::Error::new_spanned(attribute, "duplicate aldebaran dependency policy")),
            })?;

        let local = policy.is_some();

        let dependency = if local { "aldebaran-report" } else { "aldebaran" };
        let found = proc_macro_crate::crate_name(dependency)
            .map_err(|error| syn::Error::new(Span2::call_site(), format!("failed to resolve {dependency}: {error}")))?;

        let path = match found {
            proc_macro_crate::FoundCrate::Itself => quote!(crate),
            proc_macro_crate::FoundCrate::Name(name) => {
                let ident = quote::format_ident!("{}", name);

                quote!(::#ident)
            }
        };

        let runtime = if local { path } else { quote!(#path::rexport::aldebaran_report) };

        Ok(Self(runtime))
    }
}

/// Derives [`Annotated`](https://docs.rs/aldebaran-report/latest/aldebaran_report/annotated/trait.Annotated.html)
/// and [`Report`](https://docs.rs/aldebaran-report/latest/aldebaran_report/report/trait.Report.html).
///
/// Use `report` attributes to describe diagnostic headings, messages, and source
/// annotations. The derive has no requirements on [`Debug`](core::fmt::Debug),
/// [`Display`](core::fmt::Display), or [`Error`](core::error::Error) and generates
/// no implementations of those traits. A diagnostic can implement or derive
/// standard error behavior independently.
///
/// # Struct reports
///
/// A custom struct requires exactly one of `title = "text"` or `title_with = path`
/// and exactly one of `message = "text"` or `message_with = path` inside `#[report(...)]`.
/// Any function supplied through `title_with` or `message_with` receives
/// `&Self` and must return a value that coerces to `&str` for as long as that
/// borrow is valid. Text literals are used as written, without format
/// interpolation.
///
/// A unique field whose final type path segment is `Span` supplies the primary
/// target. If there is no such field, the struct must have a sole field that
/// implements `Annotated`, and the derive uses that field's target instead. It
/// checks only the path spelling, so it cannot resolve aliases. If multiple
/// fields end in `Span`, the target is ambiguous. The struct acts as its own
/// annotation and uses `Simple::Error` as its classification.
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
/// Each variant must wrap exactly one unnamed field. A custom variant needs a
/// literal `title` and either a literal `message` or a `message_with` function.
/// That function receives a reference to the payload, and its result must coerce
/// to `&str` for that borrow. A payload whose final type path segment is `Span`
/// supplies the target directly, while any other payload must implement `Annotated`.
///
/// A transparent variant takes its annotation message, report title, and target
/// from the payload, which must implement both `Annotated` and `Report`. Its
/// borrowed title must coerce to `&str` for the payload borrow. Annotation messages
/// retain their printable types. When any variant is transparent, the enum is its
/// own annotation title and supplies a borrowed printer that dispatches to the
/// active variant's message. Its printing context nests pairs of each variant's
/// message context and the remaining contexts, ending in `()`. Custom variants
/// contribute `()` as their message context.
/// The derived enum still acts as its own annotation and uses `Simple::Error`,
/// so it does not forward the payload's annotation collection or classification.
/// Variants do not support `title_with`, stored annotations, or transparent field
/// selectors.
///
/// `#[report(forward)]` on an enum selects a collection-preserving alternative.
/// Each variant must wrap exactly one report without per-variant report metadata.
/// Variants and payload fields cannot use `cfg` or `cfg_attr`, since every
/// alternative contributes to the enum's shared report contract.
/// Payload reports must share their title and kind types and their annotation
/// element type, while their annotation collection sizes may differ. The enum
/// implements `Annotations` and `Report`, forwarding the active collection and
/// classification. It leaves `Annotated` unimplemented so the blanket
/// `Annotations` implementation cannot collapse the forwarded collection to its
/// primary annotation. Nested forwarding enums preserve related annotations
/// without copying or allocating collections.
///
/// # Dependencies and type requirements
///
/// Generated implementations retain the type's declared generics and `where`
/// clauses. Ordinary report policies add no field bounds, so the declared bounds
/// must support the selected form. The forwarding policy adds payload bounds for
/// generic enums to require compatible report types. Option names must be
/// recognized, and options on a struct or variant cannot repeat or conflict.
/// Unions are unsupported. The derive does not compare targets with a source, so
/// pair each report with a source that uses the same coordinates.
#[proc_macro_derive(Report, attributes(report, aldebaran))]
#[inline]
pub fn report_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = RuntimePath::resolve(&input.attrs).and_then(|runtime| {
        let RuntimePath(runtime) = runtime;

        report::Report::new(input).map(|report| report.expand(&runtime))
    });

    match expanded {
        Ok(expanded) => TokenStream::from(expanded),
        Err(error) => TokenStream::from(error.into_compile_error()),
    }
}
