//! Derive support for Aldebaran logical assertions and classifications.
//!
//! The `Assert` derive lowers declarative assertion metadata into predicate and
//! formatting implementations. The `Choose` derive reuses that predicate to
//! return the asserted value only when the input satisfies it.

use proc_macro::TokenStream;

use proc_macro2::{Span as Span2, TokenStream as TokenStream2};

use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Expr, Field, Fields, Generics, Ident, Meta, MetaNameValue, Token, Type,
    WhereClause,
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
};

#[proc_macro_derive(Choose)]
/// Derive `Choose` from the target type's `Assert` implementation.
///
/// The generated implementation returns a clone of the asserted value when the
/// input satisfies the assertion and returns `None` otherwise.
// Procedural macro expansion is substantial and is intentionally not inlined.
#[inline]
pub fn derive_choose(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident: name_ident,
        generics,
        ..
    } = syn::parse_macro_input!(input as _);
    let (.., ty_generics, _) = generics.split_for_impl();
    let mut choose_generics = generics.clone();
    choose_generics
        .params
        .insert(0, parse_quote!(__ChooseTy: ::aldebaran_logic::prelude::Input));
    choose_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(Self: ::aldebaran_logic::prelude::Assert<__ChooseTy> + ::core::clone::Clone));
    let (impl_generics, _, where_clause) = choose_generics.split_for_impl();

    TokenStream::from(quote! {
        #[automatically_derived]
        impl #impl_generics ::aldebaran_logic::prelude::Choose<__ChooseTy> for #name_ident #ty_generics #where_clause {
            type Unit = Self;

            #[inline]
            fn choose(&self, input: __ChooseTy) -> ::core::option::Option<Self::Unit> {
                if ::aldebaran_logic::prelude::Assert::assert(self, input) {
                    ::core::option::Option::Some(::core::clone::Clone::clone(self))
                } else {
                    ::core::option::Option::None
                }
            }
        }
    })
}

#[proc_macro_derive(Assert, attributes(assert))]
/// Derive composable assertion behavior from `assert` attributes.
///
/// Struct metadata defines one assertion composition. Enum metadata defines one
/// assertion per variant and expands the corresponding predicate and formatter
/// implementations.
// Procedural macro expansion is substantial and is intentionally not inlined.
#[inline]
pub fn derive_assert(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs: mut attribute_list,
        ident: name_ident,
        generics,
        data,
        ..
    } = syn::parse_macro_input!(input as _);

    let target_code: syn::Result<TokenStream2> = 'a: {
        let target_input = match data {
            Data::Struct(DataStruct { fields: field_list, .. }) => {
                let target_clause: Option<Attribute> = 'b: {
                    for (attr_index, attr) in attribute_list.iter().enumerate() {
                        if attr.path().is_ident("assert") {
                            break 'b Some(attribute_list.swap_remove(attr_index));
                        }
                    }

                    None
                };

                let param = match target_clause.map(Param::attribute) {
                    Some(Ok(target_clause)) => target_clause,
                    Some(Err(target_error)) => break 'a Err(target_error),
                    None => break 'a Err(Error::new_spanned(&name_ident, "expected exactly one assertion attribute")),
                };

                Ok(Target::Struct(Struct {
                    param,
                    generics,
                    attribute_list,
                    name_ident,
                    field_list,
                }))
            }
            Data::Enum(DataEnum { variants, .. }) => {
                let variant_list = variants
                    .into_iter()
                    .map(
                        |syn::Variant {
                             attrs: mut attribute_list,
                             ident: name_ident,
                             fields: field_list,
                             ..
                         }| {
                            let target_clause: Option<Attribute> = 'a: {
                                let mut iter = attribute_list.iter().enumerate();

                                while let Some((attr_index, attr)) = iter.next() {
                                    if attr.path().is_ident("assert") {
                                        break 'a Some(attribute_list.swap_remove(attr_index));
                                    }
                                }

                                None
                            };

                            match target_clause.map(Param::attribute) {
                                Some(Ok(param)) => Ok(Variant {
                                    param,
                                    attribute_list,
                                    name_ident,
                                    field_list,
                                }),
                                Some(Err(target_error)) => Err(target_error),
                                None => Err(Error::new(Span2::call_site(), "expected exactly one assertion attribute")),
                            }
                        },
                    )
                    .collect::<syn::Result<Vec<Variant>>>();

                variant_list.map(|variant_list: Vec<Variant>| {
                    Target::Enum(Enum {
                        attribute_list,
                        generics,
                        name_ident,
                        variant_list,
                    })
                })
            }
            Data::Union(..) => break 'a Err(Error::new(Span2::call_site(), "union not supported")),
        };

        match target_input.map(Target::expand) {
            Ok(Ok(target_code)) => Ok(target_code),
            Ok(Err(target_error)) | Err(target_error) => Err(target_error),
        }
    };

    match target_code {
        Ok(target_code) => TokenStream::from(target_code),
        Err(target_error) => TokenStream::from(target_error.to_compile_error()),
    }
}

/// A sum-type that represents either a struct or an enum.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    /// The target struct.
    Struct(Struct),

    /// The target enum.
    Enum(Enum),
}

impl Target {
    /// Expand this target into a [`TokenStream2`].
    #[inline]
    pub fn expand(self) -> syn::Result<TokenStream2> {
        match self {
            Target::Struct(target_struct) => target_struct.expand(),
            Target::Enum(target_enum) => target_enum.expand(),
        }
    }
}

/// Validated derive state for one assertion structure.
///
/// This representation retains the assertion policy, generics, attributes, name,
/// and fields required to generate predicate and formatting implementations.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Struct {
    /// The assertion parameter for this struct.
    pub param: Param,

    /// The generic parameters of the struct.
    pub generics: Generics,

    /// The attributes of the struct.
    pub attribute_list: Vec<Attribute>,

    /// The name of the struct.
    pub name_ident: Ident,

    /// The fields of the struct.
    pub field_list: Fields,
}

impl Struct {
    pub fn expand(self) -> syn::Result<TokenStream2> {
        let Self {
            param,
            mut generics,
            name_ident,
            field_list,
            ..
        } = self;

        let WhereClause { predicates, .. } = generics.make_where_clause().clone();

        let predicates = predicates.iter();

        let (.., ty_generics, _) = generics.split_for_impl();

        let ty_params = generics.type_params();

        match param {
            Param::Conjunctive | Param::Disjunctive => {
                let operator_token = match param {
                    Param::Conjunctive => quote! { && },
                    Param::Disjunctive => quote! { || },
                    _ => unreachable!(),
                };

                let last_cond = match param {
                    Param::Conjunctive => quote! { true },
                    Param::Disjunctive => quote! { false },
                    _ => unreachable!(),
                };

                let (field_pat, assert_expr, output_expr) = {
                    let mut target_iter = field_list.iter().peekable();

                    if let Some(Field { ident: Some(..), .. }) = target_iter.peek() {
                        let ident_iter = target_iter.map(|Field { ident, .. }| ident).cloned().flatten();

                        let field_iter = ident_iter.clone();

                        let (connective, binary_connective, precedence) = match param {
                            Param::Conjunctive => (
                                quote! { ::aldebaran_logic::fmt::Connective::AND },
                                quote! { ::aldebaran_logic::fmt::BinaryConnective::AND },
                                quote! { ::aldebaran_logic::fmt::Precedence::AND },
                            ),
                            Param::Disjunctive => (
                                quote! { ::aldebaran_logic::fmt::Connective::OR },
                                quote! { ::aldebaran_logic::fmt::BinaryConnective::OR },
                                quote! { ::aldebaran_logic::fmt::Precedence::OR },
                            ),
                            _ => unreachable!(),
                        };

                        let mut output_iter = ident_iter.clone().rev();

                        let last_output = output_iter.next().into_iter();

                        let output_ident = output_iter.rev();

                        (
                            quote! {
                                Self {
                                    #(
                                        #field_iter
                                    ),*
                                }
                            },
                            quote! {
                                #(
                                    ::aldebaran_logic::prelude::Assert::<__AssertTy>::assert(#ident_iter, input) #operator_token
                                )*

                                #last_cond
                            },
                            quote! {
                                F::start(writer, #connective, target_precedence)?;

                                #(
                                    ::aldebaran_logic::prelude::Assert::<__AssertTy>::output_with::<W, F>(#output_ident, writer, #precedence)?;

                                    F::middle(writer, #binary_connective, target_precedence)?;
                                )*

                                #(
                                    ::aldebaran_logic::prelude::Assert::<__AssertTy>::output_with::<W, F>(#last_output, writer, #precedence)?;

                                    F::end(writer, #connective, target_precedence)?;
                                )*


                                Ok(())
                            },
                        )
                    } else {
                        let ident_iter = target_iter
                            .enumerate()
                            .map(|(index, Field { .. })| Ident::new(&format!("_{index}"), Span2::call_site()));

                        let field_iter = ident_iter.clone();

                        let (connective, binary_connective, precedence) = match param {
                            Param::Conjunctive => (
                                quote! { ::aldebaran_logic::fmt::Connective::AND },
                                quote! { ::aldebaran_logic::fmt::BinaryConnective::AND },
                                quote! { ::aldebaran_logic::fmt::Precedence::AND },
                            ),
                            Param::Disjunctive => (
                                quote! { ::aldebaran_logic::fmt::Connective::OR },
                                quote! { ::aldebaran_logic::fmt::BinaryConnective::OR },
                                quote! { ::aldebaran_logic::fmt::Precedence::OR },
                            ),
                            _ => unreachable!(),
                        };

                        let mut output_iter = ident_iter.clone().rev();

                        let last_output = output_iter.next().into_iter();

                        let output_ident = output_iter.rev();

                        (
                            quote! {
                                Self(
                                    #(
                                        #field_iter
                                    ),*
                                )
                            },
                            quote! {
                                #(
                                    ::aldebaran_logic::prelude::Assert::<__AssertTy>::assert(#ident_iter, input) #operator_token
                                )*

                                #last_cond
                            },
                            quote! {
                                F::start(writer, #connective, target_precedence)?;

                                #(
                                    ::aldebaran_logic::prelude::Assert::<__AssertTy>::output_with::<W, F>(#output_ident, writer, #precedence)?;

                                    F::middle(writer, #binary_connective, target_precedence)?;
                                )*

                                #(
                                    ::aldebaran_logic::prelude::Assert::<__AssertTy>::output_with::<W, F>(#last_output, writer, #precedence)?;

                                    F::end(writer, #connective, target_precedence)?;
                                )*


                                Ok(())
                            },
                        )
                    }
                };

                let field_ty = field_list.iter().map(|Field { ty, .. }| ty);

                Ok(quote! {
                    #[automatically_derived]
                    impl<__AssertTy: ::aldebaran_logic::prelude::Input, #(#ty_params),*> ::aldebaran_logic::prelude::Assert<__AssertTy> for #name_ident #ty_generics
                    where
                        #(
                            #field_ty: ::aldebaran_logic::prelude::Assert<__AssertTy>,
                        )*
                        #(
                            #predicates
                        ),*
                    {
                        #[inline]
                        fn assert(&self, input: __AssertTy) -> bool {
                            let #field_pat = self;

                            #assert_expr
                        }

                        #[inline]
                        fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: ::aldebaran_logic::fmt::Precedence) -> ::core::fmt::Result
                        where
                            W: ::core::fmt::Write,
                            F: ::aldebaran_logic::fmt::Formatter
                        {
                            let #field_pat = target_value;

                            #output_expr
                        }
                    }
                })
            }
            Param::Expr(ParamExpr { expr_value, expr_ty }) => {
                let tokens = quote! {
                    #[automatically_derived]
                    impl<__AssertTy: ::aldebaran_logic::prelude::Input, #(#ty_params),*> ::aldebaran_logic::prelude::Assert<__AssertTy> for #name_ident #ty_generics
                    where
                        #expr_ty: ::aldebaran_logic::prelude::Assert<__AssertTy>,
                        #(
                            #predicates
                        ),*
                    {
                        #[inline]
                        fn assert(&self, input: __AssertTy) -> bool {
                            let target_expr: #expr_ty = #expr_value;

                            ::aldebaran_logic::prelude::Assert::<__AssertTy>::assert(&target_expr, input)
                        }

                        #[inline]
                        fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: ::aldebaran_logic::fmt::Precedence) -> ::core::fmt::Result
                        where
                            W: ::core::fmt::Write,
                            F: ::aldebaran_logic::fmt::Formatter
                        {
                            let target_expr: #expr_ty = #expr_value;

                            ::aldebaran_logic::prelude::Assert::<__AssertTy>::output_with::<W, F>(&target_expr, writer, target_precedence)
                        }
                    }
                };

                Ok(tokens)
            }
        }
    }
}

/// Validated derive state for one assertion enum.
///
/// Each variant has already been paired with its assertion policy so expansion
/// can generate one exhaustive predicate and formatter implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Enum {
    /// The attributes of the enum.
    pub attribute_list: Vec<Attribute>,

    /// The generic parameters of the enum.
    pub generics: Generics,

    /// The name of the enum.
    pub name_ident: Ident,

    /// The variants of the enum.
    pub variant_list: Vec<Variant>,
}

impl Enum {
    pub fn expand(self) -> syn::Result<TokenStream2> {
        let Self {
            mut generics,
            name_ident,
            variant_list,
            ..
        } = self;

        let WhereClause { predicates, .. } = generics.make_where_clause().clone();

        let predicates = predicates.iter();

        let (.., ty_generics, _) = generics.split_for_impl();

        let ty_params = generics.type_params();

        let mut evaluated_arms = Vec::with_capacity(variant_list.len());

        let mut output_arms = Vec::with_capacity(variant_list.len());

        let mut assert_tys = Vec::new();

        for Variant {
            param,
            name_ident,
            field_list,
            ..
        } in variant_list
        {
            let field_pat = {
                let mut target_iter = field_list.iter().peekable();

                if let Some(Field { ident: Some(..), .. }) = target_iter.peek() {
                    let ident_iter = target_iter.map(|Field { ident, .. }| ident).cloned().flatten();

                    quote! {
                        {
                            #(
                                #ident_iter
                            ),*
                        }
                    }
                } else if let None = target_iter.peek() {
                    quote! {}
                } else {
                    let ident_iter = target_iter
                        .enumerate()
                        .map(|(index, Field { .. })| Ident::new(&format!("_{index}"), Span2::call_site()));

                    quote! {
                        (
                            #(
                                #ident_iter
                            ),*
                        )
                    }
                }
            };

            match param {
                Param::Expr(ParamExpr { expr_value, expr_ty }) => {
                    evaluated_arms.push({
                        quote! {
                            Self::#name_ident #field_pat => {
                                let target_expr: #expr_ty = #expr_value;

                                ::aldebaran_logic::prelude::Assert::<__AssertTy>::assert(&target_expr, input)
                            }
                        }
                    });

                    output_arms.push({
                        quote! {
                            Self::#name_ident #field_pat => {
                                let target_expr: #expr_ty = #expr_value;

                                ::aldebaran_logic::prelude::Assert::<__AssertTy>::output_with::<W, F>(&target_expr, writer, target_precedence)
                            }
                        }
                    });

                    assert_tys.push(expr_ty);
                }
                Param::Conjunctive | Param::Disjunctive => {
                    let operator_token = match param {
                        Param::Conjunctive => quote! { && },
                        Param::Disjunctive => quote! { || },
                        _ => unreachable!(),
                    };

                    let last_cond = match param {
                        Param::Conjunctive => quote! { true },
                        Param::Disjunctive => quote! { false },
                        _ => unreachable!(),
                    };

                    let (field_pat, assert_expr, output_expr) = {
                        let mut target_iter = field_list.iter().peekable();

                        if let Some(Field { ident: Some(..), .. }) = target_iter.peek() {
                            let ident_iter = target_iter.map(|Field { ident, .. }| ident).cloned().flatten();

                            let field_iter = ident_iter.clone();

                            let (connective, binary_connective, precedence) = match param {
                                Param::Conjunctive => (
                                    quote! { ::aldebaran_logic::fmt::Connective::AND },
                                    quote! { ::aldebaran_logic::fmt::BinaryConnective::AND },
                                    quote! { ::aldebaran_logic::fmt::Precedence::AND },
                                ),
                                Param::Disjunctive => (
                                    quote! { ::aldebaran_logic::fmt::Connective::OR },
                                    quote! { ::aldebaran_logic::fmt::BinaryConnective::OR },
                                    quote! { ::aldebaran_logic::fmt::Precedence::OR },
                                ),
                                _ => unreachable!(),
                            };

                            let mut output_iter = ident_iter.clone().rev();

                            let last_output = output_iter.next().into_iter();

                            let output_ident = output_iter.rev();

                            (
                                quote! {
                                    {
                                        #(
                                            #field_iter
                                        ),*
                                    }
                                },
                                quote! {
                                    #(
                                        ::aldebaran_logic::prelude::Assert::<__AssertTy>::assert(#ident_iter, input) #operator_token
                                    )*

                                    #last_cond
                                },
                                quote! {
                                    F::start(writer, #connective, target_precedence)?;

                                    #(
                                        ::aldebaran_logic::prelude::Assert::<__AssertTy>::output_with::<W, F>(#output_ident, writer, #precedence)?;

                                        F::middle(writer, #binary_connective, target_precedence)?;
                                    )*

                                    #(
                                        ::aldebaran_logic::prelude::Assert::<__AssertTy>::output_with::<W, F>(#last_output, writer, #precedence)?;

                                        F::end(writer, #connective, target_precedence)?;
                                    )*


                                    Ok(())
                                },
                            )
                        } else {
                            let ident_iter = target_iter
                                .enumerate()
                                .map(|(index, Field { .. })| Ident::new(&format!("_{index}"), Span2::call_site()));

                            let field_iter = ident_iter.clone();

                            let (connective, binary_connective, precedence) = match param {
                                Param::Conjunctive => (
                                    quote! { ::aldebaran_logic::fmt::Connective::AND },
                                    quote! { ::aldebaran_logic::fmt::BinaryConnective::AND },
                                    quote! { ::aldebaran_logic::fmt::Precedence::AND },
                                ),
                                Param::Disjunctive => (
                                    quote! { ::aldebaran_logic::fmt::Connective::OR },
                                    quote! { ::aldebaran_logic::fmt::BinaryConnective::OR },
                                    quote! { ::aldebaran_logic::fmt::Precedence::OR },
                                ),
                                _ => unreachable!(),
                            };

                            let mut output_iter = ident_iter.clone().rev();

                            let last_output = output_iter.next().into_iter();

                            let output_ident = output_iter.rev();

                            (
                                quote! {
                                    (
                                        #(
                                            #field_iter
                                        ),*
                                    )
                                },
                                quote! {
                                    #(
                                        ::aldebaran_logic::prelude::Assert::<__AssertTy>::assert(#ident_iter, input) #operator_token
                                    )*

                                    #last_cond
                                },
                                quote! {
                                    F::start(writer, #connective, target_precedence)?;

                                    #(
                                        ::aldebaran_logic::prelude::Assert::<__AssertTy>::output_with::<W, F>(#output_ident, writer, #precedence)?;

                                        F::middle(writer, #binary_connective, target_precedence)?;
                                    )*

                                    #(
                                        ::aldebaran_logic::prelude::Assert::<__AssertTy>::output_with::<W, F>(#last_output, writer, #precedence)?;

                                        F::end(writer, #connective, target_precedence)?;
                                    )*


                                    Ok(())
                                },
                            )
                        }
                    };

                    let field_ty = field_list.iter().map(|Field { ty, .. }| ty);

                    evaluated_arms.push({
                        quote! {
                            Self::#name_ident #field_pat => {
                                #assert_expr
                            }
                        }
                    });

                    output_arms.push({
                        quote! {
                            Self::#name_ident #field_pat => {
                                #output_expr
                            }
                        }
                    });

                    assert_tys.extend(field_ty.cloned());
                }
            }
        }

        Ok(quote! {
            #[automatically_derived]
            impl<__AssertTy: ::aldebaran_logic::prelude::Input, #(#ty_params),*> ::aldebaran_logic::prelude::Assert<__AssertTy> for #name_ident #ty_generics
            where
                #(
                    #assert_tys: ::aldebaran_logic::prelude::Assert<__AssertTy>,
                )*
                #(
                    #predicates,
                )*
            {
                #[inline]
                fn assert(&self, input: __AssertTy) -> bool {
                    match self {
                        #(
                            #evaluated_arms
                        )*
                    }
                }

                #[inline]
                fn output_with<W, F>(target_value: &Self, writer: &mut W, target_precedence: ::aldebaran_logic::fmt::Precedence) -> ::core::fmt::Result
                where
                    W: ::core::fmt::Write,
                    F: ::aldebaran_logic::fmt::Formatter
                {
                    match target_value {
                        #(
                            #output_arms
                        )*
                    }
                }
            }
        })
    }
}

/// One validated assertion enum variant used during derive expansion.
///
/// The stored policy determines how the variant fields participate in the
/// generated assertion while the remaining syntax is retained for pattern output.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Variant {
    /// The assertion parameter for this variant.
    param: Param,

    /// The attributes of the variant.
    pub attribute_list: Vec<Attribute>,

    /// The name of the variant.
    pub name_ident: Ident,

    /// The fields of the variant.
    pub field_list: Fields,
}

/// An assertion parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Param {
    /// An assertion dictated by an arbitrary expression.
    Expr(ParamExpr),

    /// A conjunctive assertion.
    Conjunctive,

    /// A disjunctive assertion.
    Disjunctive,
}

impl Parse for Param {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Ident) {
            let target_mode: Ident = input.parse()?;

            match target_mode.to_string().as_str() {
                "conjunctive" => Ok(Param::Conjunctive),
                "disjunctive" => Ok(Param::Disjunctive),
                _ => Err(Error::new_spanned(target_mode, "expected either `conjunctive` or `disjunctive`")),
            }
        } else if lookahead.peek(Token![use]) {
            let _ = input.parse::<Token![use]>()?;

            let expr_value = input.parse::<Expr>()?;

            let _: Token![type] = input.parse()?;

            let expr_ty = input.parse::<Type>()?;

            Ok(Param::Expr(ParamExpr { expr_value, expr_ty }))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Param {
    /// Parse the target attribute into a [`Param`].
    pub fn attribute(target_attr: Attribute) -> syn::Result<Self> {
        let attr_span = target_attr.span();

        let attr_path = target_attr.path();

        let is_valid = attr_path.is_ident("assert");

        if is_valid {
            let Attribute { meta, .. } = target_attr;

            match meta {
                Meta::List(meta_list) => meta_list.parse_args::<Self>(),
                Meta::NameValue(MetaNameValue { value, .. }) => {
                    Err(Error::new_spanned(value, "expected either a name-value or a list attribute"))
                }
                Meta::Path(path) => Err(Error::new_spanned(path, "expected either a name-value or a list attribute")),
            }
        } else {
            Err(Error::new(attr_span, "expected an assertion attribute"))
        }
    }
}

/// A parameter that indicates an assertion expression.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParamExpr {
    /// The expression itself.
    pub expr_value: Expr,

    /// The type of the expression.
    pub expr_ty: Type,
}
