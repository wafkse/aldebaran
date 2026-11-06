//! Annotation-preserving sums of concrete reports.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, DataEnum, Error, Fields, Generics, Ident, Meta, Result, Token, Type, parse_quote, punctuated::Punctuated};

/// A variant and the report type it contains.
#[derive(Debug)]
// NOTE(invariant): PayloadArity = 1
struct Alternative {
    /// Variant constructor name.
    ident: Ident,

    /// Wrapped report type.
    payload: Type,
}

/// Transparent alternatives sharing annotation element, title, and kind types.
#[derive(Debug)]
// NOTE(invariant): Alternatives.Length > 0
pub(super) struct Forward {
    /// Alternatives in declaration order.
    alternatives: Vec<Alternative>,
}

impl Forward {
    /// Validate the exclusive enum policy and its payloads.
    pub(super) fn new(data: DataEnum, attributes: Vec<Attribute>) -> Result<Self> {
        let report_attributes = attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("report"))
            .collect::<Vec<_>>();
        let report_attribute = match report_attributes.as_slice() {
            [attribute] => *attribute,
            _ => return Err(Error::new_spanned(data.enum_token, "forward excludes other enum report policies")),
        };
        let metadata = report_attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

        let is_only_forward = metadata.len() == 1
            && metadata
                .first()
                .is_some_and(|metadata| matches!(metadata, Meta::Path(path) if path.is_ident("forward")));

        if !is_only_forward {
            return Err(Error::new_spanned(
                report_attribute,
                "enum-level report metadata only accepts the standalone `forward` option",
            ));
        }

        let mut alternatives = Vec::new();

        for variant in data.variants {
            let syn::Variant { attrs, ident, fields, .. } = variant;

            if let Some(attribute) = attrs
                .iter()
                .find(|attribute| attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr"))
            {
                return Err(Error::new_spanned(attribute, "forward alternatives cannot use cfg or cfg_attr"));
            }

            if attrs.iter().any(|attribute| attribute.path().is_ident("report")) {
                return Err(Error::new_spanned(
                    ident,
                    "forward alternatives do not select individual report policies",
                ));
            }

            let payload = match fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    let Some(field) = fields.unnamed.into_iter().next() else {
                        return Err(Error::new_spanned(&ident, "a forwarded alternative wraps exactly one report"));
                    };

                    if let Some(attribute) = field
                        .attrs
                        .iter()
                        .find(|attribute| attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr"))
                    {
                        return Err(Error::new_spanned(attribute, "forward payloads cannot use cfg or cfg_attr"));
                    }

                    Some(field.ty)
                }
                _ => None,
            };
            let payload = payload.ok_or_else(|| Error::new_spanned(&ident, "a forwarded alternative wraps exactly one report"))?;

            alternatives.push(Alternative { ident, payload });
        }

        if alternatives.is_empty() {
            return Err(Error::new_spanned(data.enum_token, "forward requires a report alternative"));
        }

        Ok(Self { alternatives })
    }

    /// Forward every annotation and the active alternative's report heading.
    pub(super) fn expand(&self, ident: &Ident, generics: &Generics, runtime: &TokenStream) -> TokenStream {
        let base_type = &self.alternatives[0].payload;
        let mut bounds = generics.clone();

        if !generics.params.is_empty() {
            bounds
                .make_where_clause()
                .predicates
                .push(parse_quote!(#base_type: #runtime::report::Report));

            for alternative in self.alternatives.iter().skip(1) {
                let payload = &alternative.payload;
                let report_bound = parse_quote!(
                    #payload: #runtime::report::Report<
                        Title = <#base_type as #runtime::report::Report>::Title,
                        Kind = <#base_type as #runtime::report::Report>::Kind,
                    >
                );
                let annotation_bound = parse_quote!(
                    <#payload as #runtime::report::Report>::Annotations:
                        #runtime::annotated::Annotations<
                            Annotation = <<#base_type as #runtime::report::Report>::Annotations
                                as #runtime::annotated::Annotations>::Annotation,
                        >
                );

                let predicates = &mut bounds.make_where_clause().predicates;
                predicates.push(report_bound);
                predicates.push(annotation_bound);
            }
        }

        let (impl_generics, _, where_clause) = bounds.split_for_impl();
        let (_, type_generics, _) = generics.split_for_impl();

        let annotation_arms = self.alternatives.iter().map(|alternative| {
            let ident = &alternative.ident;

            quote! {
                Self::#ident(error) => {
                    #runtime::annotated::Annotations::list(
                        #runtime::report::Report::annotations(error),
                    )
                },
            }
        });
        let title_arms = self.alternatives.iter().map(|alternative| {
            let ident = &alternative.ident;

            quote! {
                Self::#ident(error) => #runtime::report::Report::title(error),
            }
        });
        let kind_arms = self.alternatives.iter().map(|alternative| {
            let ident = &alternative.ident;

            quote! {
                Self::#ident(error) => #runtime::report::Report::kind(error),
            }
        });

        quote! {
            #[automatically_derived]
            impl #impl_generics #runtime::annotated::Annotations for #ident #type_generics #where_clause {
                type Annotation = <<#base_type as #runtime::report::Report>::Annotations
                    as #runtime::annotated::Annotations>::Annotation;

                #[inline]
                fn list(&self) -> Option<#runtime::reexport::RefNonempty<'_, Self::Annotation>> {
                    match self {
                        #(#annotation_arms)*
                    }
                }
            }

            #[automatically_derived]
            impl #impl_generics #runtime::report::Report for #ident #type_generics #where_clause {
                type Title = <#base_type as #runtime::report::Report>::Title;
                type Kind = <#base_type as #runtime::report::Report>::Kind;
                type Annotations = Self;

                #[inline]
                fn title(&self) -> &Self::Title {
                    match self {
                        #(#title_arms)*
                    }
                }

                #[inline]
                fn kind(&self) -> &Self::Kind {
                    match self {
                        #(#kind_arms)*
                    }
                }

                #[inline]
                fn annotations(&self) -> &Self::Annotations {
                    self
                }
            }
        }
    }
}
