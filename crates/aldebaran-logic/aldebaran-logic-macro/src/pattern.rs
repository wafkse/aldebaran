//! Derived structural expectations over a concrete input domain.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Error, Fields, Ident, LitStr, Pat, Result, Token, Type,
    parse::{Parse, ParseStream},
};

/// Concrete input and named pattern expectations.
#[derive(Debug)]
// NOTE(invariant): Input.Data = Enum && Rules.Count = Input.Enum.Variants.Count && ForAll((Rule, Variant) In Zip(Rules, Input.Enum.Variants)): Rule.Name = Variant.Name && Variant.Fields = Unit
pub(super) struct Pattern {
    /// Declaration receiving the assertion implementation.
    input: DeriveInput,

    /// Domain matched by the assertion.
    domain: Type,

    /// Alternatives in declaration order.
    rules: Vec<(Ident, Rule, Vec<Attribute>)>,
}

/// A Rust pattern and its expectation text.
#[derive(Debug)]
struct Rule {
    /// Accepted input shape.
    pattern: Pat,

    /// Description used by assertion formatters.
    description: LitStr,
}

impl Parse for Rule {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let word: Ident = input.parse()?;

        if word != "matches" {
            return Err(Error::new_spanned(word, "expected matches(pattern) as a description"));
        }

        let content;

        syn::parenthesized!(content in input);

        let pattern = content.call(Pat::parse_multi)?;

        input.parse::<Token![as]>()?;

        let description = input.parse()?;

        Ok(Self { pattern, description })
    }
}

impl Pattern {
    /// Whether the declaration selects concrete input pattern assertions.
    #[inline]
    pub(super) fn selected(attributes: &[Attribute]) -> bool {
        attributes
            .iter()
            .any(|attribute| attribute.path().is_ident("assert") && attribute.parse_args::<Domain>().is_ok())
    }

    /// Validate the input domain and every named alternative.
    pub(super) fn new(input: DeriveInput) -> Result<Self> {
        let domain_list = input
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident("assert"))
            .map(|attribute| attribute.parse_args::<Domain>())
            .collect::<Result<Vec<_>>>()?;

        let domain = match domain_list.as_slice() {
            [Domain(domain)] => domain.clone(),
            _ => return Err(Error::new_spanned(&input, "expected exactly one input domain")),
        };

        let variant_punctuated = match &input.data {
            Data::Enum(data) => &data.variants,
            _ => {
                return Err(Error::new_spanned(
                    &input,
                    "pattern assertions require an enum of named expectations",
                ));
            }
        };

        let mut rules = Vec::new();

        for target_variant in variant_punctuated {
            if target_variant.fields != Fields::Unit {
                return Err(Error::new_spanned(target_variant, "pattern expectations must be unit variants"));
            }

            let attribute_list = target_variant
                .attrs
                .iter()
                .filter(|attribute| attribute.path().is_ident("assert"))
                .collect::<Vec<_>>();

            let rule = match attribute_list.as_slice() {
                [attribute] => attribute.parse_args()?,
                _ => return Err(Error::new_spanned(target_variant, "expected exactly one pattern assertion")),
            };

            let cfg = target_variant
                .attrs
                .iter()
                .filter(|attribute| attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr"))
                .cloned()
                .collect();

            rules.push((target_variant.ident.clone(), rule, cfg));
        }

        Ok(Self { input, domain, rules })
    }

    /// Generate typed matching and allocation-free descriptions.
    pub(super) fn expand(self, runtime: &TokenStream) -> TokenStream {
        let Self { input, domain, rules } = self;

        let DeriveInput { ident, generics, .. } = input;

        let (impl_generic, type_generic, where_clause) = generics.split_for_impl();

        let predicate_iter = rules
            .iter()
            .map(|(name, Rule { pattern, .. }, cfg)| quote!(#(#cfg)* Self::#name => matches!(input, #pattern),));

        let descriptions = rules
            .iter()
            .map(|(name, Rule { description, .. }, cfg)| quote!(#(#cfg)* Self::#name => writer.write_str(#description),));

        quote! {
            #[automatically_derived]
            impl #impl_generic #runtime::prelude::Assert<#domain> for #ident #type_generic #where_clause {
                #[inline]
                fn assert(&self, input: #domain) -> bool {
                    match self {
                        #(#predicate_iter)*
                    }
                }

                #[inline]
                fn output_with<W, F>(target_value: &Self, writer: &mut W, _: #runtime::fmt::Precedence) -> ::core::fmt::Result
                where
                    W: ::core::fmt::Write,
                    F: #runtime::fmt::Formatter
                {
                    match target_value {
                        #(#descriptions)*
                    }
                }
            }
        }
    }
}

/// Selects a concrete input type without introducing a token tag domain.
struct Domain(Type);

impl Parse for Domain {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let word: Ident = input.parse()?;

        if word != "input" {
            return Err(Error::new_spanned(word, "expected input(Type)"));
        }

        let content;

        syn::parenthesized!(content in input);

        Ok(Self(content.parse()?))
    }
}
