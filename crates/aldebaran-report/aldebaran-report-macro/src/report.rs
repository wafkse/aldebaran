//! Parsing and expansion of allocation-free report policies.
//!
//! See [`Report`] for further information.

use darling::{FromMeta, ast::NestedMeta};
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::ext::IdentExt;
use syn::fold::{self, Fold};
use syn::{Attribute, Data, DeriveInput, Fields, Ident, LitStr, Member, Path, Type};

/// Validated report derive input ready for policy expansion.
///
/// Construction classifies the input as a structure or enum and validates its
/// report metadata before code generation. Expansion consumes this representation
/// to emit the runtime `Annotated` and `Report` implementations.
#[derive(Debug)]
// NOTE(invariant): Body In ValidReportPolicies(Ident, DeclaredFields)
pub struct Report {
    /// Name of the diagnostic type.
    ident: Ident,

    /// Generic parameters of the diagnostic type.
    generics: syn::Generics,

    /// Validated diagnostic shape.
    body: Body,
}

impl Report {
    /// Parse and validate the derive input.
    #[inline]
    pub fn new(input: DeriveInput) -> syn::Result<Self> {
        let DeriveInput {
            attrs,
            ident,
            generics,
            data,
            ..
        } = input;

        let body = match data {
            Data::Struct(data) => Body::Struct(Box::new(Structure::new(attrs, data.fields, &ident)?)),
            Data::Enum(data) => data
                .variants
                .into_iter()
                .map(Variant::new)
                .collect::<syn::Result<Vec<_>>>()
                .map(Body::Enum)?,
            Data::Union(_) => Err(syn::Error::new_spanned(ident.clone(), "Report does not support unions"))?,
        };

        Ok(Self { ident, generics, body })
    }

    /// Generate allocation-free annotation and report implementations.
    #[inline]
    pub fn expand(self, runtime: &TokenStream) -> TokenStream {
        let Self { ident, generics, body } = self;

        match body {
            Body::Struct(structure) => structure.expand(&ident, &generics, runtime),
            Body::Enum(variants) => Body::expand_enum(&ident, &generics, &variants, runtime),
        }
    }
}

/// Keeps `Self` tied to the input enum when syntax moves into a printer impl.
#[derive(Debug)]
// NOTE(invariant): Target = OriginalEnum(DeclaredGenericArguments)
struct InputSelf(
    /// Original enum type, including its declared generic arguments.
    syn::TypePath,
);

impl InputSelf {
    /// Qualify a leading `Self` path with the original enum type.
    fn qualify(&self, path: syn::TypePath) -> syn::TypePath {
        match path {
            syn::TypePath { attrs, qself: None, path }
                if path.leading_colon.is_none() && path.segments.first().is_some_and(|segment| segment.ident == "Self") =>
            {
                let Self(target) = self;

                let mut qualified = if path.segments.len() == 1 {
                    target.clone()
                } else {
                    let suffix = Path {
                        leading_colon: None,
                        segments: path.segments.into_iter().skip(1).collect(),
                    };

                    syn::parse_quote!(<#target>::#suffix)
                };

                qualified.attrs = attrs;

                qualified
            }
            path => path,
        }
    }
}

impl Fold for InputSelf {
    fn fold_type_path(&mut self, path: syn::TypePath) -> syn::TypePath {
        let path = fold::fold_type_path(self, path);

        self.qualify(path)
    }

    fn fold_expr_path(&mut self, path: syn::ExprPath) -> syn::ExprPath {
        let syn::ExprPath { attrs, qself, path } = fold::fold_expr_path(self, path);

        let path = syn::TypePath { attrs, qself, path };

        let syn::TypePath { attrs, qself, path } = self.qualify(path);

        syn::ExprPath { attrs, qself, path }
    }
}

/// Validated report shape.
#[derive(Debug)]
enum Body {
    /// Report structure generation state.
    Struct(
        /// Validated report structure policy.
        Box<Structure>,
    ),

    /// Per-alternative report policies.
    Enum(
        /// Validated policies for each enum alternative.
        Vec<Variant>,
    ),
}

impl Body {
    /// Generate the report enum implementation.
    fn expand_enum(ident: &Ident, generics: &syn::Generics, variants: &[Variant], runtime: &TokenStream) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let typed = variants.iter().any(|variant| matches!(variant.policy, Policy::Transparent));

        let (annotation_title, message, adapter) = if typed {
            let adapter = Self::expand_message(ident, generics, variants, runtime);

            (quote!(Self), quote!(self), adapter)
        } else {
            let arms = variants.iter().map(|variant| variant.message_arm(runtime));
            let message = quote!(match self { #(#arms)* });

            (quote!(str), message, TokenStream::new())
        };

        let target = variants.iter().map(|variant| variant.target_arm(runtime));
        let title = variants.iter().map(|variant| variant.title_arm(runtime));

        quote! {
            #adapter

            impl #impl_generics #runtime::annotated::Annotated for #ident #ty_generics #where_clause {
                type Title = #annotation_title;

                #[inline]
                fn message(&self) -> &Self::Title {
                    #message
                }

                #[inline]
                fn target(&self) -> #runtime::reexport::Span {
                    match self {
                        #(#target)*
                    }
                }
            }

            impl #impl_generics #runtime::report::Report for #ident #ty_generics #where_clause {
                type Title = str;
                type Kind = #runtime::report::kind::Simple;
                type Annotations = Self;

                #[inline]
                fn kind(&self) -> &Self::Kind {
                    &#runtime::report::kind::Simple::Error
                }

                #[inline]
                fn title(&self) -> &Self::Title {
                    match self {
                        #(#title)*
                    }
                }

                #[inline]
                fn annotations(&self) -> &Self::Annotations {
                    self
                }
            }
        }
    }

    /// Generate a borrowed printable view of heterogeneous annotation messages.
    ///
    /// The enum itself supplies the annotation title, while the local adapter
    /// dispatches printing to the active payload. Independent context tuple
    /// entries preserve each payload's printing requirements. Nested pairs keep
    /// default contexts available regardless of the number of variants.
    fn expand_message(ident: &Ident, generics: &syn::Generics, variants: &[Variant], runtime: &TokenStream) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let mut input_self = InputSelf(syn::parse_quote!(#ident #ty_generics));
        let printer_generics = input_self.fold_generics(generics.clone());

        let (printer_impl_generics, _, printer_where_clause) = printer_generics.split_for_impl();

        let ident_path = ident;
        let message_ident = Self::fresh_ident(generics, &format!("__{}Message", ident.unraw()));
        let writer_ident = Self::fresh_ident(generics, &format!("__{}Writer", ident.unraw()));
        let contexts = variants.iter().map(|variant| match variant {
            Variant {
                ty,
                policy: Policy::Transparent,
                ..
            } => {
                quote!(<<#ty as #runtime::annotated::Annotated>::Title as #runtime::annotated::title::Title>::Context)
            }
            Variant {
                policy: Policy::Custom { .. },
                ..
            } => quote!(()),
        });
        let context = contexts.rev().fold(quote!(()), |tail, head| quote!((#head, #tail)));
        let arms = variants.iter().enumerate().map(|(index, variant)| {
            let Variant { ident, policy, cfg, .. } = variant;

            let context = (0..index).fold(quote!(context), |context, _| quote!(#context.1));
            let message = match policy {
                Policy::Transparent => quote!(#runtime::annotated::Annotated::message(error)),
                Policy::Custom {
                    message: Message::With(path),
                    ..
                } => {
                    let path = syn::ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: path.clone(),
                    };
                    let path = input_self.fold_expr_path(path);

                    quote!(#path(error))
                }
                Policy::Custom { message, .. } => message.expression(quote!(error)),
            };

            quote! {
                #(#cfg)*
                #ident_path::#ident(error) => #runtime::reexport::Print::print_with_context(
                    &#runtime::annotated::title::Title::content(#message),
                    writer,
                    &#context.0,
                ),
            }
        });

        quote! {
            const _: () = {
                struct #message_ident<'a, T: ?Sized>(&'a T);

                impl #printer_impl_generics #runtime::reexport::Print for #message_ident<'_, #ident #ty_generics> #printer_where_clause {
                    type Context = <#ident #ty_generics as #runtime::annotated::title::Title>::Context;

                    #[inline]
                    fn print_with_context<#writer_ident>(&self, writer: &mut #writer_ident, context: &Self::Context) -> ::core::fmt::Result
                    where
                        #writer_ident: ::core::fmt::Write,
                    {
                        let Self(error) = self;

                        match error {
                            #(#arms)*
                        }
                    }
                }

                impl #impl_generics #runtime::annotated::title::Title for #ident #ty_generics #where_clause {
                    type Context = #context;

                    #[inline]
                    fn content(&self) -> impl #runtime::reexport::Print<Context = Self::Context> {
                        #message_ident(self)
                    }
                }
            };
        }
    }

    /// Choose a generated type name that cannot shadow an input parameter.
    fn fresh_ident(generics: &syn::Generics, stem: &str) -> Ident {
        let mut ident = quote::format_ident!("{stem}");

        while generics.params.iter().any(|parameter| match parameter {
            syn::GenericParam::Type(parameter) => parameter.ident == ident,
            syn::GenericParam::Const(parameter) => parameter.ident == ident,
            syn::GenericParam::Lifetime(_) => false,
        }) {
            ident = quote::format_ident!("{ident}_");
        }

        ident
    }
}

/// Validated generation state for a report structure.
///
/// The structure owns the reporting policy selected from its attributes
/// and fields. Expansion delegates to that policy after target ambiguity and
/// incompatible metadata combinations have already been rejected.
#[derive(Debug)]
// NOTE(invariant): Policy In CompatibleReportPolicies(ResolvedMembers)
struct Structure {
    /// Validated report generation policy.
    policy: StructurePolicy,
}

impl Structure {
    /// Validate the report structure.
    fn new(attributes: Vec<Attribute>, fields: Fields, ident: &Ident) -> syn::Result<Self> {
        let members = Self::members(fields);
        let policy = StructurePolicy::parse(&attributes, &members, ident)?;

        Ok(Self { policy })
    }

    /// Normalize structure fields into members and types.
    fn members(fields: Fields) -> Vec<(Member, Type)> {
        match fields {
            Fields::Named(fields) => fields
                .named
                .into_iter()
                .filter_map(|field| field.ident.map(|ident| (Member::Named(ident), field.ty)))
                .collect(),
            Fields::Unnamed(fields) => fields
                .unnamed
                .into_iter()
                .enumerate()
                .map(|(index, field)| (Member::Unnamed(index.into()), field.ty))
                .collect(),
            Fields::Unit => Vec::new(),
        }
    }

    /// Treat a type as a source span only when its final path segment is `Span`.
    /// This check cannot resolve aliases or inspect the underlying type.
    fn is_span(ty: &Type) -> bool {
        match ty {
            Type::Path(path) => path.path.segments.last().map(|segment| segment.ident == "Span").unwrap_or(false),
            _ => false,
        }
    }

    /// Find the declared member and preserve its type.
    fn member(members: &[(Member, Type)], selected: &Member, ident: &Ident) -> syn::Result<TargetMember> {
        members
            .iter()
            .find(|(member, _)| member == selected)
            .map(|(member, ty)| TargetMember::new(member.clone(), ty.clone()))
            .ok_or_else(|| syn::Error::new_spanned(ident, "report member does not exist"))
    }

    /// Use the only field whose type path ends in `Span`, or the sole field's
    /// annotation target when no such `Span` field exists. Return an error when
    /// neither choice is unique.
    fn target(members: &[(Member, Type)], ident: &Ident) -> syn::Result<TargetMember> {
        let spans = members
            .iter()
            .filter(|(_, ty)| Self::is_span(ty))
            .map(|(member, ty)| TargetMember::Span(member.clone(), ty.clone()))
            .collect::<Vec<_>>();

        match spans.as_slice() {
            [target] => Ok(target.clone()),
            [] => match members {
                [(member, ty)] => Ok(TargetMember::Annotated(member.clone(), ty.clone())),
                _ => Err(syn::Error::new_spanned(
                    ident,
                    "Report requires one unambiguous Span or wrapped annotated field",
                )),
            },
            _ => Err(syn::Error::new_spanned(
                ident,
                "Report cannot infer a primary target from multiple Span fields",
            )),
        }
    }

    /// Generate the struct's report implementations using its declared generics
    /// and bounds.
    fn expand(&self, ident: &Ident, generics: &syn::Generics, runtime: &TokenStream) -> TokenStream {
        let Self { policy } = self;

        policy.expand(ident, generics, runtime)
    }
}

/// Generation policy for a report structure.
///
/// A policy determines whether behavior is forwarded to a wrapped report, built
/// from a primary source target, or backed by an existing annotation collection.
/// Each variant contains everything needed to generate both reporting traits.
#[derive(Debug)]
enum StructurePolicy {
    /// Forward every diagnostic facet to the wrapped report field.
    Transparent(
        /// Wrapped report member receiving every forwarded operation.
        TargetMember,
    ),

    /// Build the primary annotation from a selected source target.
    Custom {
        /// Diagnostic title policy.
        title: TitlePolicy,

        /// Primary annotation message policy.
        message: Message,

        /// Member establishing the primary source target.
        target: TargetMember,
    },

    /// Reuse a statically nonempty inline annotation collection.
    Stored {
        /// Diagnostic title policy.
        title: TitlePolicy,

        /// Optional message for the enclosing annotation. When absent, it uses
        /// the collection's primary message.
        message: Option<Message>,

        /// Member storing the annotation collection.
        annotations: TargetMember,
    },
}

impl StructurePolicy {
    /// Parse the structure report policy.
    fn parse(attributes: &[Attribute], members: &[(Member, Type)], ident: &Ident) -> syn::Result<Self> {
        let declarations = Declarations::parse(attributes)?;

        let Declarations {
            transparent,
            title,
            title_with,
            message,
            message_with,
            annotations,
        } = declarations;

        match (transparent, title, title_with, message, message_with, annotations) {
            (Some(selected), None, None, None, None, None) => {
                let target = match selected {
                    Some(member) => Structure::member(members, &member, ident)?,
                    None => match members {
                        [(member, ty)] => TargetMember::new(member.clone(), ty.clone()),
                        _ => Err(syn::Error::new_spanned(
                            ident,
                            "transparent Report on a multi-field structure requires a target member",
                        ))?,
                    },
                };

                match target {
                    TargetMember::Span(_, _) => Err(syn::Error::new_spanned(
                        ident,
                        "transparent Report requires one wrapped report field",
                    )),
                    target => Ok(Self::Transparent(target)),
                }
            }
            (None, title, title_with, message, message_with, None) => {
                let title = TitlePolicy::resolve(title, title_with, ident)?;
                let message = Message::resolve(message, message_with, ident)?;
                let target = Structure::target(members, ident)?;

                Ok(Self::Custom { title, message, target })
            }
            (None, title, title_with, message, message_with, Some(member)) => {
                let title = TitlePolicy::resolve(title, title_with, ident)?;
                let message = Message::optional(message, message_with, ident)?;
                let annotations = Structure::member(members, &member, ident)?;

                Ok(Self::Stored {
                    title,
                    message,
                    annotations,
                })
            }
            _ => Err(syn::Error::new_spanned(ident, "invalid Report metadata combination")),
        }
    }

    /// Generate implementations for the validated structure policy.
    fn expand(&self, ident: &Ident, generics: &syn::Generics, runtime: &TokenStream) -> TokenStream {
        match self {
            Self::Transparent(target) => Self::expand_transparent(ident, generics, runtime, target),
            Self::Custom { title, message, target } => Self::expand_custom(ident, generics, runtime, title, message, target),
            Self::Stored {
                title,
                message,
                annotations,
            } => Self::expand_stored(ident, generics, runtime, title, message.as_ref(), annotations),
        }
    }

    /// Generate the custom primary-annotation implementation.
    fn expand_custom(
        ident: &Ident,
        generics: &syn::Generics,
        runtime: &TokenStream,
        title: &TitlePolicy,
        message: &Message,
        target: &TargetMember,
    ) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let title = title.expression(quote!(self));
        let message = message.expression(quote!(self));
        let target = target.expression(runtime);

        quote! {
            impl #impl_generics #runtime::annotated::Annotated for #ident #ty_generics #where_clause {
                type Title = str;

                #[inline]
                fn message(&self) -> &Self::Title {
                    #message
                }

                #[inline]
                fn target(&self) -> #runtime::reexport::Span {
                    #target
                }
            }

            impl #impl_generics #runtime::report::Report for #ident #ty_generics #where_clause {
                type Title = str;
                type Kind = #runtime::report::kind::Simple;
                type Annotations = Self;

                #[inline]
                fn kind(&self) -> &Self::Kind {
                    &#runtime::report::kind::Simple::Error
                }

                #[inline]
                fn title(&self) -> &Self::Title {
                    #title
                }

                #[inline]
                fn annotations(&self) -> &Self::Annotations {
                    self
                }
            }
        }
    }

    /// Generate the transparent forwarding implementation.
    fn expand_transparent(ident: &Ident, generics: &syn::Generics, runtime: &TokenStream, target: &TargetMember) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let inner = target.reference();
        let ty = target.ty();

        quote! {
            impl #impl_generics #runtime::annotated::Annotated for #ident #ty_generics #where_clause {
                type Title = <#ty as #runtime::annotated::Annotated>::Title;

                #[inline]
                fn message(&self) -> &Self::Title {
                    #runtime::annotated::Annotated::message(#inner)
                }

                #[inline]
                fn target(&self) -> #runtime::reexport::Span {
                    #runtime::annotated::Annotated::target(#inner)
                }
            }

            impl #impl_generics #runtime::report::Report for #ident #ty_generics #where_clause {
                type Title = <#ty as #runtime::report::Report>::Title;
                type Kind = <#ty as #runtime::report::Report>::Kind;
                type Annotations = <#ty as #runtime::report::Report>::Annotations;

                #[inline]
                fn kind(&self) -> &Self::Kind {
                    #runtime::report::Report::kind(#inner)
                }

                #[inline]
                fn title(&self) -> &Self::Title {
                    #runtime::report::Report::title(#inner)
                }

                #[inline]
                fn annotations(&self) -> &Self::Annotations {
                    #runtime::report::Report::annotations(#inner)
                }
            }
        }
    }

    /// Generate a report backed by the stored nonempty annotation collection.
    fn expand_stored(
        ident: &Ident,
        generics: &syn::Generics,
        runtime: &TokenStream,
        title: &TitlePolicy,
        message: Option<&Message>,
        annotations: &TargetMember,
    ) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let title = title.expression(quote!(self));
        let annotations_ref = annotations.reference();
        let annotations_ty = annotations.ty();

        let (annotation_title, annotation_message) = match message {
            Some(message) => {
                let message = message.expression(quote!(self));

                (quote!(str), quote!(#message))
            }
            None => (
                quote!(<<#annotations_ty as #runtime::annotated::Annotations>::Annotation as #runtime::annotated::Annotated>::Title),
                quote!({
                    let primary = #runtime::annotated::PrimaryAnnotations::primary(#annotations_ref);

                    #runtime::annotated::Annotated::message(primary)
                }),
            ),
        };

        quote! {
            impl #impl_generics #runtime::annotated::Annotated for #ident #ty_generics #where_clause {
                type Title = #annotation_title;

                #[inline]
                fn message(&self) -> &Self::Title {
                    #annotation_message
                }

                #[inline]
                fn target(&self) -> #runtime::reexport::Span {
                    let primary = #runtime::annotated::PrimaryAnnotations::primary(#annotations_ref);

                    #runtime::annotated::Annotated::target(primary)
                }
            }

            impl #impl_generics #runtime::report::Report for #ident #ty_generics #where_clause {
                type Title = str;
                type Kind = #runtime::report::kind::Simple;
                type Annotations = #annotations_ty;

                #[inline]
                fn kind(&self) -> &Self::Kind {
                    &#runtime::report::kind::Simple::Error
                }

                #[inline]
                fn title(&self) -> &Self::Title {
                    #title
                }

                #[inline]
                fn annotations(&self) -> &Self::Annotations {
                    #annotations_ref
                }
            }
        }
    }
}

/// Report options parsed from `#[report(...)]` attributes.
#[derive(Debug, Default)]
// NOTE(invariant): None.
struct Declarations {
    /// Optional transparent forwarding target. `Some(None)` means infer the sole field.
    transparent: Option<Option<Member>>,

    /// Optional static title.
    title: Option<LitStr>,

    /// Optional title projection function.
    title_with: Option<Path>,

    /// Optional static primary message.
    message: Option<LitStr>,

    /// Optional primary-message projection function.
    message_with: Option<Path>,

    /// Optional stored annotation member.
    annotations: Option<Member>,
}

impl Declarations {
    /// Parse report declarations from attributes.
    fn parse(attributes: &[Attribute]) -> syn::Result<Self> {
        let mut transparent = None;
        let mut standard = Vec::new();

        for attribute in attributes.iter().filter(|attribute| attribute.path().is_ident("report")) {
            let meta_list = match &attribute.meta {
                syn::Meta::List(meta_list) => meta_list,
                meta => return Err(syn::Error::new_spanned(meta, "report metadata must use list form")),
            };

            for nested in NestedMeta::parse_meta_list(meta_list.tokens.clone())? {
                let nested_meta = match &nested {
                    NestedMeta::Meta(meta) if meta.path().is_ident("transparent") => Some(meta),
                    _ => None,
                };

                match nested_meta {
                    Some(meta) => {
                        if transparent.is_some() {
                            return Err(syn::Error::new_spanned(meta, "duplicate transparent report policy"));
                        }

                        let selected = match meta {
                            syn::Meta::Path(_) => None,
                            syn::Meta::List(meta_list) => Some(syn::parse2(meta_list.tokens.clone())?),
                            syn::Meta::NameValue(meta) => {
                                return Err(syn::Error::new_spanned(meta, "transparent does not accept a value"));
                            }
                        };

                        transparent = Some(selected);
                    }
                    None => standard.push(nested),
                }
            }
        }

        let standard = StandardDeclarations::from_list(&standard).map_err(syn::Error::from)?;

        let StandardDeclarations {
            title,
            title_with,
            message,
            message_with,
            annotations,
        } = standard;

        Ok(Self {
            transparent,
            title,
            title_with,
            message,
            message_with,
            annotations: annotations.map(|MemberArgument(member)| member),
        })
    }
}

/// Standard report metadata parsed from `#[report(...)]` attributes.
#[derive(Debug, Default, FromMeta)]
// NOTE(invariant): None.
struct StandardDeclarations {
    /// Static title text.
    title: Option<LitStr>,

    /// Function path that returns a title borrowed from the report value.
    title_with: Option<Path>,

    /// Static primary annotation message.
    message: Option<LitStr>,

    /// Function path that returns a primary message from a report value or variant payload.
    message_with: Option<Path>,

    /// Field containing stored annotations.
    annotations: Option<MemberArgument>,
}

/// A field selector parsed from the value of `annotations` metadata.
#[derive(Debug)]
// NOTE(invariant): None.
struct MemberArgument(Member);

impl FromMeta for MemberArgument {
    fn from_expr(value: &syn::Expr) -> darling::Result<Self> {
        syn::parse2(value.to_token_stream()).map(Self).map_err(darling::Error::custom)
    }
}

/// Diagnostic title policy.
#[derive(Debug)]
enum TitlePolicy {
    /// Static diagnostic title.
    Static(
        /// Literal emitted as the diagnostic title.
        LitStr,
    ),

    /// Get the report title by calling a function.
    With(
        /// Function path that returns a title borrowed from the value.
        Path,
    ),
}

impl TitlePolicy {
    /// Require either a literal title or `title_with`, but not both.
    fn resolve(title: Option<LitStr>, title_with: Option<Path>, ident: &Ident) -> syn::Result<Self> {
        match (title, title_with) {
            (Some(title), None) => Ok(Self::Static(title)),
            (None, Some(path)) => Ok(Self::With(path)),
            (None, None) => Err(syn::Error::new_spanned(ident, "Report requires title or title_with metadata")),
            (Some(_), Some(_)) => Err(syn::Error::new_spanned(ident, "title and title_with are mutually exclusive")),
        }
    }

    /// Generate the title expression.
    fn expression(&self, subject: TokenStream) -> TokenStream {
        match self {
            Self::Static(title) => quote!(#title),
            Self::With(path) => quote!(#path(#subject)),
        }
    }
}

/// A selected structure member, stored with its declared type.
#[derive(Debug, Clone)]
enum TargetMember {
    /// Member stores a source span directly.
    Span(
        /// Structure member containing the source span.
        Member,
        /// Declared type of the selected member.
        Type,
    ),

    /// Member whose target is obtained through annotation traits.
    Annotated(
        /// Structure member used to get a target through annotation traits.
        Member,
        /// Declared type of the selected member.
        Type,
    ),
}

impl TargetMember {
    /// Record whether the member is a `Span` or gets its target through annotations.
    fn new(member: Member, ty: Type) -> Self {
        if Structure::is_span(&ty) {
            Self::Span(member, ty)
        } else {
            Self::Annotated(member, ty)
        }
    }

    /// Generate a borrow of the selected member.
    fn reference(&self) -> TokenStream {
        let member = match self {
            Self::Span(member, _) | Self::Annotated(member, _) => member,
        };

        quote!(&self.#member)
    }

    /// Retrieve the selected member type.
    const fn ty(&self) -> &Type {
        match self {
            Self::Span(_, ty) | Self::Annotated(_, ty) => ty,
        }
    }

    /// Generate an expression that reads the field's `Span` or calls
    /// `Annotated::target` to get its target.
    fn expression(&self, runtime: &TokenStream) -> TokenStream {
        match self {
            Self::Span(member, _) => quote!(self.#member),
            Self::Annotated(member, _) => quote!(#runtime::annotated::Annotated::target(&self.#member)),
        }
    }
}

/// Primary annotation message policy.
#[derive(Debug)]
enum Message {
    /// Static annotation text.
    Static(
        /// Literal emitted as the primary annotation message.
        LitStr,
    ),

    /// Get annotation text by calling a function on the report value or variant payload.
    With(
        /// Function path that returns a borrowed annotation message.
        Path,
    ),
}

impl Message {
    /// Accept either `message`, `message_with`, or neither, and reject declarations
    /// that set both.
    fn optional(message: Option<LitStr>, message_with: Option<Path>, ident: &Ident) -> syn::Result<Option<Self>> {
        match (message, message_with) {
            (Some(message), None) => Ok(Some(Self::Static(message))),
            (None, Some(path)) => Ok(Some(Self::With(path))),
            (None, None) => Ok(None),
            (Some(_), Some(_)) => Err(syn::Error::new_spanned(ident, "message and message_with are mutually exclusive")),
        }
    }

    /// Require either `message` or `message_with`, but not both.
    fn resolve(message: Option<LitStr>, message_with: Option<Path>, ident: &Ident) -> syn::Result<Self> {
        match (message, message_with) {
            (Some(message), None) => Ok(Self::Static(message)),
            (None, Some(path)) => Ok(Self::With(path)),
            (None, None) => Err(syn::Error::new_spanned(ident, "Report requires message or message_with metadata")),
            (Some(_), Some(_)) => Err(syn::Error::new_spanned(ident, "message and message_with are mutually exclusive")),
        }
    }

    /// Generate the message expression.
    fn expression(&self, subject: TokenStream) -> TokenStream {
        match self {
            Self::Static(message) => quote!(#message),
            Self::With(path) => quote!(#path(#subject)),
        }
    }
}

/// Validated report enum alternative.
#[derive(Debug)]
// NOTE(invariant): UnnamedFields(Ident) = [Ty] And Policy In ValidReportPolicies(Ty)
struct Variant {
    /// Variant constructor name.
    ident: Ident,

    /// Wrapped diagnostic type.
    ty: Type,

    /// Report wording and forwarding policy.
    policy: Policy,

    /// Configuration attributes copied to generated match arms.
    cfg: Vec<Attribute>,
}

impl Variant {
    /// Validate the report enum alternative.
    fn new(variant: syn::Variant) -> syn::Result<Self> {
        let syn::Variant { attrs, ident, fields, .. } = variant;

        let ty = match fields {
            Fields::Unnamed(fields) => {
                let mut fields = fields.unnamed.into_iter();
                let first = fields.next();
                let second = fields.next();

                match (first, second) {
                    (Some(field), None) => field.ty,
                    _ => Err(syn::Error::new_spanned(
                        ident.clone(),
                        "Report enum alternatives must wrap exactly one unnamed field",
                    ))?,
                }
            }
            _ => Err(syn::Error::new_spanned(
                ident.clone(),
                "Report enum alternatives must be tuple variants",
            ))?,
        };
        let policy = Policy::parse(&attrs, &ident)?;
        let cfg = attrs.into_iter().filter(|attribute| attribute.path().is_ident("cfg")).collect();

        Ok(Self { ident, ty, policy, cfg })
    }

    /// Generate the message match arm.
    fn message_arm(&self, runtime: &TokenStream) -> TokenStream {
        let Self { ident, policy, cfg, .. } = self;

        let message = match policy {
            Policy::Transparent => quote!(#runtime::annotated::Annotated::message(error)),
            Policy::Custom { message, .. } => message.expression(quote!(error)),
        };

        quote! {
            #(#cfg)*
            Self::#ident(error) => #message,
        }
    }

    /// Generate the target match arm.
    fn target_arm(&self, runtime: &TokenStream) -> TokenStream {
        let Self { ident, ty, cfg, .. } = self;

        let target = if Structure::is_span(ty) {
            quote!(*error)
        } else {
            quote!(#runtime::annotated::Annotated::target(error))
        };

        quote! {
            #(#cfg)*
            Self::#ident(error) => #target,
        }
    }

    /// Generate the title match arm.
    fn title_arm(&self, runtime: &TokenStream) -> TokenStream {
        let Self { ident, policy, cfg, .. } = self;

        let title = match policy {
            Policy::Transparent => quote!(#runtime::report::Report::title(error)),
            Policy::Custom { title, .. } => quote!(#title),
        };

        quote! {
            #(#cfg)*
            Self::#ident(error) => #title,
        }
    }
}

/// Report wording and forwarding policy.
#[derive(Debug)]
enum Policy {
    /// Reuse the wrapped report's annotation message and report title.
    Transparent,

    /// Use a literal title and a separately selected annotation message.
    Custom {
        /// Static diagnostic title.
        title: LitStr,

        /// Primary annotation message policy.
        message: Message,
    },
}

impl Policy {
    /// Parse report metadata from an item or enum alternative.
    fn parse(attributes: &[Attribute], ident: &Ident) -> syn::Result<Self> {
        let Declarations {
            transparent,
            title,
            title_with,
            message,
            message_with,
            annotations,
        } = Declarations::parse(attributes)?;

        let transparent = match transparent {
            None => false,
            Some(None) => true,
            Some(Some(member)) => {
                return Err(syn::Error::new_spanned(
                    member,
                    "enum report alternatives do not select transparent fields",
                ));
            }
        };

        match (transparent, title, title_with, message, message_with, annotations) {
            (true, None, None, None, None, None) => Ok(Self::Transparent),
            (true, _, _, _, _, _) => Err(syn::Error::new_spanned(
                ident,
                "transparent report metadata cannot be combined with custom wording",
            )),
            (false, Some(title), None, Some(message), None, None) => Ok(Self::Custom {
                title,
                message: Message::Static(message),
            }),
            (false, Some(title), None, None, Some(path), None) => Ok(Self::Custom {
                title,
                message: Message::With(path),
            }),
            (false, None, _, _, _, _) => Err(syn::Error::new_spanned(ident, "Report requires title metadata")),
            (false, Some(_), _, Some(_), Some(_), _) => {
                Err(syn::Error::new_spanned(ident, "message and message_with are mutually exclusive"))
            }
            (false, Some(_), _, Some(_), None, _) => {
                Err(syn::Error::new_spanned(ident, "Report does not accept this metadata combination"))
            }
            (false, Some(_), _, None, None, _) => Err(syn::Error::new_spanned(ident, "Report requires message or message_with metadata")),
            (false, Some(_), Some(_), _, _, _) => Err(syn::Error::new_spanned(ident, "Report alternatives require a static title")),
            (false, Some(_), None, None, Some(_), Some(_)) => {
                Err(syn::Error::new_spanned(ident, "enum report alternatives do not store annotations"))
            }
        }
    }
}
