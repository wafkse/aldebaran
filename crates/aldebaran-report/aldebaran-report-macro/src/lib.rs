//! Procedural entry point for deriving Aldebaran diagnostic reports.
//!
//! The macro resolves the runtime crate path, validates report metadata, and
//! emits both ordinary error behavior and Aldebaran report implementations.

mod report;

use proc_macro::TokenStream;

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

/// Runtime path used by generated report implementations.
#[derive(Debug)]
struct RuntimePath(
    /// Tokens naming the report runtime visible from generated code.
    TokenStream2,
);

impl RuntimePath {
    /// Resolve either the direct runtime crate or the public facade.
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

    /// Convert a direct dependency result into a Rust path.
    fn direct(found: FoundCrate) -> TokenStream2 {
        match found {
            FoundCrate::Itself => quote!(crate),
            FoundCrate::Name(name) => {
                let ident = format_ident!("{}", name);

                quote!(::#ident)
            }
        }
    }

    /// Convert a facade dependency result into its report module path.
    fn facade(found: FoundCrate) -> TokenStream2 {
        match found {
            FoundCrate::Itself => quote!(crate::report),
            FoundCrate::Name(name) => {
                let ident = format_ident!("{}", name);

                quote!(::#ident::report)
            }
        }
    }

    /// Consume the resolved path.
    fn into_tokens(self) -> TokenStream2 {
        let Self(tokens) = self;

        tokens
    }
}

/// Derive ordinary error behavior and an allocation-free Aldebaran report.
#[proc_macro_derive(Report, attributes(error, report))]
// Procedural macro expansion is substantial and is intentionally not inlined.
#[inline]
pub fn report_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = RuntimePath::resolve().and_then(|runtime| {
        let runtime = runtime.into_tokens();
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
