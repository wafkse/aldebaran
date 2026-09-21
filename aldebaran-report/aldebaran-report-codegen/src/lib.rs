#![forbid(unsafe_code, missing_docs, rustdoc::all, clippy::all)]
//! Code generation for allocation-free Aldebaran reports.
//!
//! The crate validates report metadata independently from procedural macro
//! plumbing, then emits `Annotated` and `Report` implementations against a
//! caller-provided runtime path. Ordinary error behavior is generated alongside it.

pub mod report;

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Generate ordinary error behavior and an allocation-free Aldebaran report.
///
/// # Errors
///
/// Returns a diagnostic when either Fack error semantics or Aldebaran report
/// semantics are invalid for the supplied type.
#[inline]
pub fn generate(input: DeriveInput, runtime: &TokenStream) -> syn::Result<TokenStream> {
    let error = fack_codegen::generate(&input)?;
    let report = report::Report::new(input)?.expand(runtime);

    Ok(quote! {
        #error
        #report
    })
}

pub mod prelude {
    //! A prelude for the `aldebaran-report-codegen` crate.
    //!
    //! This simply re-exports the most commonly used items in the
    //! `aldebaran-report-codegen` crate.

    pub use crate::report::Report;
}
