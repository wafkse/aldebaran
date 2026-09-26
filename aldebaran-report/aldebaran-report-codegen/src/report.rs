//! Parsing and expansion of allocation-free report policies.
//!
//! Derive input is normalized into explicit structure or enum policies before
//! any tokens are emitted. Expansion then projects titles, annotation messages,
//! source targets, and stored annotation collections into runtime trait impls.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, LitStr, Member, Path, Type};

/// Validated report derive input ready for policy expansion.
///
/// Construction classifies the input as a structure or enum and validates its
/// report metadata before code generation. Expansion consumes this representation
/// to emit the runtime `Annotated` and `Report` implementations.
#[derive(Debug)]
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
            Data::Struct(data) => Body::Struct(Structure::new(attrs, data.fields, &ident)?),
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

/// Validated report shape.
#[derive(Debug)]
enum Body {
    /// Report structure generation state.
    Struct(
        /// Validated report structure policy.
        Structure,
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
        let message = variants.iter().map(|variant| variant.message_arm(runtime));
        let target = variants.iter().map(|variant| variant.target_arm(runtime));
        let title = variants.iter().map(|variant| variant.title_arm(runtime));

        quote! {
            impl #impl_generics #runtime::annotated::Annotated for #ident #ty_generics #where_clause {
                type Title = str;

                #[inline]
                fn message(&self) -> &Self::Title {
                    match self {
                        #(#message)*
                    }
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
}

/// Validated generation state for a report structure.
///
/// The structure owns the reporting policy selected from its attributes
/// and fields. Expansion delegates to that policy after target ambiguity and
/// incompatible metadata combinations have already been rejected.
#[derive(Debug)]
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

    /// Determine whether a type is syntactically a source span.
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

    /// Infer an unambiguous source target.
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

    /// Generate the report structure implementation.
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

        /// Optional primary-message projection overriding the stored annotation title type.
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

/// Parsed structure-level report declarations.
#[derive(Debug, Default)]
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
    /// Parse report declarations from structure attributes.
    fn parse(attributes: &[Attribute]) -> syn::Result<Self> {
        let mut declarations = Self::default();
        let reports = attributes.iter().filter(|attribute| attribute.path().is_ident("report"));

        for attribute in reports {
            attribute.parse_nested_meta(|meta| {
                let Self {
                    transparent,
                    title,
                    title_with,
                    message,
                    message_with,
                    annotations,
                } = &mut declarations;
                let is_transparent = meta.path.is_ident("transparent");
                let is_title = meta.path.is_ident("title");
                let is_title_with = meta.path.is_ident("title_with");
                let is_message = meta.path.is_ident("message");
                let is_message_with = meta.path.is_ident("message_with");
                let is_annotations = meta.path.is_ident("annotations");

                match (is_transparent, is_title, is_title_with, is_message, is_message_with, is_annotations) {
                    (true, false, false, false, false, false) => {
                        let selected = if meta.input.peek(syn::token::Paren) {
                            let content;
                            syn::parenthesized!(content in meta.input);

                            Some(content.parse::<Member>()?)
                        } else {
                            None
                        };
                        *transparent = Some(selected);
                        Ok(())
                    }
                    (false, true, false, false, false, false) => {
                        *title = Some(meta.value()?.parse::<LitStr>()?);
                        Ok(())
                    }
                    (false, false, true, false, false, false) => {
                        *title_with = Some(meta.value()?.parse::<Path>()?);
                        Ok(())
                    }
                    (false, false, false, true, false, false) => {
                        *message = Some(meta.value()?.parse::<LitStr>()?);
                        Ok(())
                    }
                    (false, false, false, false, true, false) => {
                        *message_with = Some(meta.value()?.parse::<Path>()?);
                        Ok(())
                    }
                    (false, false, false, false, false, true) => {
                        *annotations = Some(meta.value()?.parse::<Member>()?);
                        Ok(())
                    }
                    _ => Err(meta.error("expected transparent, title, title_with, message, message_with, or annotations")),
                }
            })?;
        }

        Ok(declarations)
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

    /// Function used to project a borrowed title from the error.
    With(
        /// Function path used to project a borrowed diagnostic title.
        Path,
    ),
}

impl TitlePolicy {
    /// Resolve the exclusive title declaration.
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

/// Source target selected from a structure member.
#[derive(Debug, Clone)]
enum TargetMember {
    /// Member stores a source span directly.
    Span(
        /// Structure member containing the source span.
        Member,
        /// Declared type of the selected member.
        Type,
    ),

    /// Member supplies its target through Annotated.
    Annotated(
        /// Structure member implementing the annotation contract.
        Member,
        /// Declared type of the selected member.
        Type,
    ),
}

impl TargetMember {
    /// Construct the member classification.
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

    /// Generate the source-span expression.
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

    /// Function used to project annotation text from the error.
    With(
        /// Function path used to project a borrowed annotation message.
        Path,
    ),
}

impl Message {
    /// Resolve an optional exclusive message declaration.
    fn optional(message: Option<LitStr>, message_with: Option<Path>, ident: &Ident) -> syn::Result<Option<Self>> {
        match (message, message_with) {
            (Some(message), None) => Ok(Some(Self::Static(message))),
            (None, Some(path)) => Ok(Some(Self::With(path))),
            (None, None) => Ok(None),
            (Some(_), Some(_)) => Err(syn::Error::new_spanned(ident, "message and message_with are mutually exclusive")),
        }
    }

    /// Resolve the required exclusive message declaration.
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
    /// Forward wording to the wrapped report.
    Transparent,

    /// Use explicit static title and message policy.
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
        let mut transparent = false;
        let mut title = None;
        let mut message = None;
        let mut message_with = None;
        let reports = attributes.iter().filter(|attribute| attribute.path().is_ident("report"));

        for attribute in reports {
            attribute.parse_nested_meta(|meta| {
                let is_transparent = meta.path.is_ident("transparent");
                let is_title = meta.path.is_ident("title");
                let is_message = meta.path.is_ident("message");
                let is_message_with = meta.path.is_ident("message_with");

                match (is_transparent, is_title, is_message, is_message_with) {
                    (true, false, false, false) => {
                        transparent = true;
                        Ok(())
                    }
                    (false, true, false, false) => {
                        title = Some(meta.value()?.parse::<LitStr>()?);
                        Ok(())
                    }
                    (false, false, true, false) => {
                        message = Some(meta.value()?.parse::<LitStr>()?);
                        Ok(())
                    }
                    (false, false, false, true) => {
                        message_with = Some(meta.value()?.parse::<Path>()?);
                        Ok(())
                    }
                    _ => Err(meta.error("expected transparent, title, message, or message_with")),
                }
            })?;
        }

        match (transparent, title, message, message_with) {
            (true, None, None, None) => Ok(Self::Transparent),
            (true, _, _, _) => Err(syn::Error::new_spanned(
                ident,
                "transparent report metadata cannot be combined with custom wording",
            )),
            (false, Some(title), Some(message), None) => Ok(Self::Custom {
                title,
                message: Message::Static(message),
            }),
            (false, Some(title), None, Some(path)) => Ok(Self::Custom {
                title,
                message: Message::With(path),
            }),
            (false, None, _, _) => Err(syn::Error::new_spanned(ident, "Report requires title metadata")),
            (false, Some(_), Some(_), Some(_)) => Err(syn::Error::new_spanned(ident, "message and message_with are mutually exclusive")),
            (false, Some(_), None, None) => Err(syn::Error::new_spanned(ident, "Report requires message or message_with metadata")),
        }
    }
}
