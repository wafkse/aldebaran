//! Procedural entry point for deriving Aldebaran diagnostic reports.
//!
//! The macro resolves the runtime crate path, delegates semantic validation and
//! expansion to `aldebaran-report-codegen`, and returns compile errors at the
//! original derive site when report metadata is invalid.

use proc_macro::TokenStream;

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

/// Runtime path used by generated report implementations.
#[derive(Debug)]
struct RuntimePath(TokenStream2);

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

    /// Convert one direct dependency result into a Rust path.
    fn direct(found: FoundCrate) -> TokenStream2 {
        match found {
            FoundCrate::Itself => quote!(crate),
            FoundCrate::Name(name) => {
                let ident = format_ident!("{}", name);

                quote!(::#ident)
            }
        }
    }

    /// Convert one facade dependency result into its report module path.
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
        aldebaran_report_codegen::generate(input, &runtime)
    });

    match expanded {
        Ok(expanded) => TokenStream::from(expanded),
        Err(error) => TokenStream::from(error.into_compile_error()),
    }
}
