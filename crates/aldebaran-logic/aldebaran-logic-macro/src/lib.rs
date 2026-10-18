#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;

use proc_macro2::{Span as Span2, TokenStream as TokenStream2};

use darling::{FromMeta, ast::NestedMeta};
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Expr, Field, Fields, Generics, Ident, Meta, Token, Type,
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
};

#[proc_macro_derive(Choose)]
/// Derive [`Choose`](https://docs.rs/aldebaran-logic/latest/aldebaran_logic/choose/trait.Choose.html)
/// for a type implementing [`Assert`](https://docs.rs/aldebaran-logic/latest/aldebaran_logic/assert/trait.Assert.html)
/// and [`Clone`]. Accepted inputs return a clone of the value.
#[inline]
pub fn derive_choose(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident: name_ident,
        generics,
        ..
    } = syn::parse_macro_input!(input as DeriveInput);

    let (.., ty_generics, _) = generics.split_for_impl();

    let mut choose_generics = generics.clone();

    choose_generics
        .params
        .push(parse_quote!(__ChooseTy: ::aldebaran_logic::prelude::Input));

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

/// Derive assertion behavior from `assert` attributes on a struct or enum.
///
/// `#[assert(conjunctive)]` requires every field predicate and
/// `#[assert(disjunctive)]` accepts any field predicate, with each field type
/// implementing [`Assert`](https://docs.rs/aldebaran-logic/latest/aldebaran_logic/assert/trait.Assert.html)
/// for the same input. Field predicates have declaration-order, short-circuit
/// semantics, while descriptions include all fields. Empty conjunctions accept
/// and empty disjunctions reject.
///
/// `#[assert(use EXPR type TY)]` instead assigns acceptance and descriptions to
/// an expression of type `TY`, which must implement `Assert` for the input.
/// The expression is evaluated separately for each predicate or formatting call,
/// and unused field types need not implement `Assert`.
///
/// A struct or each enum variant requires exactly one supported attribute.
/// Enums use the active variant's rule, and unions are unsupported.
#[proc_macro_derive(Assert, attributes(assert))]
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
                let target_clause = match Param::take_attribute(&mut attribute_list) {
                    Ok(target_clause) => target_clause,
                    Err(target_error) => break 'a Err(target_error),
                };

                let param = match target_clause.map(Param::attribute) {
                    Some(Ok(target_clause)) => target_clause,
                    Some(Err(target_error)) => break 'a Err(target_error),
                    None => break 'a Err(Error::new_spanned(&name_ident, "expected exactly one assertion attribute")),
                };

                Ok(Target::Struct(Box::new(Struct {
                    param,
                    generics,
                    attribute_list,
                    name_ident,
                    field_list,
                })))
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
                            let target_clause = Param::take_attribute(&mut attribute_list)?;

                            match target_clause.map(Param::attribute) {
                                Some(Ok(param)) => Ok(Variant {
                                    param,
                                    attribute_list,
                                    name_ident,
                                    field_list,
                                }),
                                Some(Err(target_error)) => Err(target_error),
                                None => Err(Error::new_spanned(&name_ident, "expected exactly one assertion attribute")),
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

/// An assertion declaration whose rule belongs to a struct or its enum variants.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    /// A declaration with a shared rule for its fields.
    Struct(Box<Struct>),

    /// A declaration whose active variant determines its rule.
    Enum(Enum),
}

impl Target {
    /// Produce acceptance and descriptions governed by the declaration's rule.
    #[inline]
    pub fn expand(self) -> syn::Result<TokenStream2> {
        match self {
            Target::Struct(target_struct) => target_struct.expand(),
            Target::Enum(target_enum) => target_enum.expand(),
        }
    }
}

/// A struct's predicate rule and the declaration it applies to.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Struct {
    /// Rule applied to the struct's fields.
    pub param: Param,

    /// Parameters and bounds required wherever this declaration is used.
    pub generics: Generics,

    /// Non-assertion metadata supplied with the declaration.
    pub attribute_list: Vec<Attribute>,

    /// Identity of the type receiving the assertion contract.
    pub name_ident: Ident,

    /// Predicate operands for field-based rules, including their declaration order.
    pub field_list: Fields,
}

impl Struct {
    /// Apply the struct's rule to acceptance and descriptions for a shared input type.
    pub fn expand(self) -> syn::Result<TokenStream2> {
        let Self {
            param,
            generics,
            name_ident,
            field_list,
            ..
        } = self;

        let (.., ty_generics, _) = generics.split_for_impl();

        let mut assert_generics = generics.clone();

        assert_generics
            .params
            .push(parse_quote!(__AssertTy: ::aldebaran_logic::prelude::Input));

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
                    let target_iter = field_list.iter();

                    if matches!(&field_list, Fields::Named(_)) {
                        let ident_iter = target_iter.flat_map(|Field { ident, .. }| ident).cloned();

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
                                )*

                                F::end(writer, #connective, target_precedence)?;

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
                            match &field_list {
                                Fields::Unit => quote! { Self },
                                _ => quote! {
                                    Self(
                                        #(
                                            #field_iter
                                        ),*
                                    )
                                },
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
                                )*

                                F::end(writer, #connective, target_precedence)?;

                                Ok(())
                            },
                        )
                    }
                };

                for Field { ty, .. } in &field_list {
                    assert_generics
                        .make_where_clause()
                        .predicates
                        .push(parse_quote!(#ty: ::aldebaran_logic::prelude::Assert<__AssertTy>));
                }

                let (impl_generics, _, where_clause) = assert_generics.split_for_impl();

                Ok(quote! {
                    #[automatically_derived]
                    impl #impl_generics ::aldebaran_logic::prelude::Assert<__AssertTy> for #name_ident #ty_generics #where_clause
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
            Param::Expr(param_expr) => {
                let ParamExpr { expr_value, expr_ty } = *param_expr;

                assert_generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote!(#expr_ty: ::aldebaran_logic::prelude::Assert<__AssertTy>));

                let (impl_generics, _, where_clause) = assert_generics.split_for_impl();

                let tokens = quote! {
                    #[automatically_derived]
                    impl #impl_generics ::aldebaran_logic::prelude::Assert<__AssertTy> for #name_ident #ty_generics #where_clause
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

/// Assertion alternatives whose active variant supplies the acceptance rule.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Enum {
    /// Non-assertion metadata supplied with the declaration.
    pub attribute_list: Vec<Attribute>,

    /// Parameters and bounds shared by all assertion alternatives.
    pub generics: Generics,

    /// Identity of the type receiving the assertion contract.
    pub name_ident: Ident,

    /// Variants and their assertion rules.
    pub variant_list: Vec<Variant>,
}

impl Enum {
    /// Preserve each alternative's acceptance and description rules for a shared input type.
    pub fn expand(self) -> syn::Result<TokenStream2> {
        let Self {
            generics,
            name_ident,
            variant_list,
            ..
        } = self;

        let (.., ty_generics, _) = generics.split_for_impl();

        let mut assert_generics = generics.clone();

        assert_generics
            .params
            .push(parse_quote!(__AssertTy: ::aldebaran_logic::prelude::Input));

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
                let target_iter = field_list.iter();

                if matches!(&field_list, Fields::Named(_)) {
                    let ident_iter = target_iter.flat_map(|Field { ident, .. }| ident).cloned();

                    quote! {
                        {
                            #(
                                #ident_iter
                            ),*
                        }
                    }
                } else if matches!(&field_list, Fields::Unit) {
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
                Param::Expr(param_expr) => {
                    let ParamExpr { expr_value, expr_ty } = *param_expr;

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
                        let target_iter = field_list.iter();

                        if matches!(&field_list, Fields::Named(_)) {
                            let ident_iter = target_iter.flat_map(|Field { ident, .. }| ident).cloned();

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
                                    )*

                                    F::end(writer, #connective, target_precedence)?;

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
                                match &field_list {
                                    Fields::Unit => quote! {},
                                    _ => quote! {
                                        (
                                            #(
                                                #field_iter
                                            ),*
                                        )
                                    },
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
                                    )*

                                    F::end(writer, #connective, target_precedence)?;

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

        for ty in assert_tys {
            assert_generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::aldebaran_logic::prelude::Assert<__AssertTy>));
        }

        let (impl_generics, _, where_clause) = assert_generics.split_for_impl();

        Ok(quote! {
            #[automatically_derived]
            impl #impl_generics ::aldebaran_logic::prelude::Assert<__AssertTy> for #name_ident #ty_generics #where_clause
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

/// An assertion alternative with its own field-based or delegated rule.
#[derive(Debug, Clone, PartialEq, Eq)]
// NOTE(invariant): None.
struct Variant {
    /// Rule applied to this variant's fields.
    param: Param,

    /// Non-assertion metadata supplied with this alternative.
    pub attribute_list: Vec<Attribute>,

    /// Identity selecting this alternative's assertion rule.
    pub name_ident: Ident,

    /// Ordered predicate operands or bindings available to a delegated expression.
    pub field_list: Fields,
}

/// Rule selected by an `assert` attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Param {
    /// Delegate to an assertion expression.
    Expr(Box<ParamExpr>),

    /// Require all field assertions to accept.
    Conjunctive,

    /// Require any field assertion to accept.
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
            let _: Token![use] = input.parse()?;

            let expr_value = input.parse::<Expr>()?;

            let _: Token![type] = input.parse()?;

            let expr_ty = input.parse::<Type>()?;

            Ok(Param::Expr(Box::new(ParamExpr { expr_value, expr_ty })))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Param {
    /// An optional assertion rule, with duplicate rules rejected and unrelated metadata retained.
    fn take_attribute(attributes: &mut Vec<Attribute>) -> syn::Result<Option<Attribute>> {
        let mut found = None;
        let mut duplicate = None;
        let mut retained = Vec::with_capacity(attributes.len());

        for attribute in attributes.drain(..) {
            if attribute.path().is_ident("assert") {
                if found.is_some() {
                    if duplicate.is_none() {
                        duplicate = Some(attribute);
                    }
                } else {
                    found = Some(attribute);
                }
            } else {
                retained.push(attribute);
            }
        }

        *attributes = retained;

        match duplicate {
            Some(attribute) => Err(Error::new_spanned(attribute, "duplicate assertion attribute")),
            None => Ok(found),
        }
    }

    /// Accept a delegated expression or exactly one enabled field-composition policy.
    pub fn attribute(target_attr: Attribute) -> syn::Result<Self> {
        let attr_span = target_attr.span();

        if !target_attr.path().is_ident("assert") {
            return Err(Error::new(attr_span, "expected an assertion attribute"));
        }

        let meta_tokens = match &target_attr.meta {
            Meta::List(meta_list) => meta_list.tokens.clone(),
            meta => return Err(Error::new_spanned(meta, "assertion metadata must use list form")),
        };

        let custom = matches!(
            meta_tokens.clone().into_iter().next(),
            Some(proc_macro2::TokenTree::Ident(ident)) if ident == "use"
        );

        if custom {
            syn::parse2::<Self>(meta_tokens)
        } else {
            let nested = NestedMeta::parse_meta_list(meta_tokens)?;
            let metadata = AssertionMetadata::from_list(&nested).map_err(Error::from)?;

            let conjunctive = metadata.conjunctive == Some(true);
            let disjunctive = metadata.disjunctive == Some(true);

            match (conjunctive, disjunctive) {
                (true, false) => Ok(Self::Conjunctive),
                (false, true) => Ok(Self::Disjunctive),
                (true, true) => Err(Error::new(attr_span, "assertion policy cannot be both conjunctive and disjunctive")),
                (false, false) => Err(Error::new(attr_span, "expected conjunctive or disjunctive assertion metadata")),
            }
        }
    }
}

/// Field-composition options requiring exactly one enabled policy for acceptance.
#[derive(Debug, Default, FromMeta)]
// NOTE(invariant): None.
struct AssertionMetadata {
    /// Requires every field predicate when enabled, excluding disjunction.
    conjunctive: Option<bool>,

    /// Accepts any field predicate when enabled, excluding conjunction.
    disjunctive: Option<bool>,
}

/// A delegated assertion expression and type.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParamExpr {
    /// Predicate expression shared by acceptance and diagnostic descriptions.
    pub expr_value: Expr,

    /// Declared expression type carrying the required input-specific assertion contract.
    pub expr_ty: Type,
}

#[cfg(test)]
mod tests {
    use syn::{Attribute, parse_quote};

    use super::{Param, ParamExpr};

    #[test]
    fn parses_declarative_and_custom_assertion_syntax() {
        let conjunctive: Attribute = parse_quote!(#[assert(conjunctive)]);
        assert!(matches!(Param::attribute(conjunctive), Ok(Param::Conjunctive)));

        let custom: Attribute = parse_quote!(#[assert(use Or::one(A, B) type Or<A, B>)]);
        assert!(matches!(Param::attribute(custom), Ok(Param::Expr(expression)) if matches!(*expression, ParamExpr { .. })));
    }

    #[test]
    fn rejects_malformed_duplicate_and_conflicting_assertion_metadata() {
        let malformed: Attribute = parse_quote!(#[assert(unknown)]);
        assert!(Param::attribute(malformed).is_err());

        let duplicate_key: Attribute = parse_quote!(#[assert(conjunctive, conjunctive)]);
        assert!(Param::attribute(duplicate_key).is_err());

        let conflicting: Attribute = parse_quote!(#[assert(conjunctive, disjunctive)]);
        assert!(Param::attribute(conflicting).is_err());

        let mut duplicate_attributes = vec![parse_quote!(#[assert(conjunctive)]), parse_quote!(#[assert(disjunctive)])];
        assert!(Param::take_attribute(&mut duplicate_attributes).is_err());
    }
}
