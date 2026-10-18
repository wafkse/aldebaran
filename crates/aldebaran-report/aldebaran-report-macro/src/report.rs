//! Diagnostic policies admitted by the `Report` derive. Structs support inferred
//! targets, stored annotations, or report forwarding, while enum alternatives
//! share `str` messages and `Simple::Error` classification.

use darling::{FromMeta, ast::NestedMeta};
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, LitStr, Member, Path, Type};

/// A diagnostic declaration with unambiguous report metadata and resolved field
/// selectors. Required runtime traits remain obligations of the declared field types.
#[derive(Debug)]
// NOTE(invariant): Body In ValidReportPolicies(Ident, DeclaredFields)
pub struct Report {
    /// Diagnostic type receiving the annotation and report contracts.
    ident: Ident,

    /// Caller-declared parameters and bounds preserved by the report implementations.
    generics: syn::Generics,

    /// Report policy compatible with this diagnostic's field shape and metadata.
    body: Body,
}

impl Report {
    /// Accept a struct or enum declaration only when its report policies and field
    /// selectors are coherent, rejecting unions and ambiguous targets.
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

    /// Express the selected diagnostic contracts through `runtime`, preserving the
    /// declaration's generics and bounds.
    #[inline]
    pub fn expand(self, runtime: &TokenStream) -> TokenStream {
        let Self { ident, generics, body } = self;

        match body {
            Body::Struct(structure) => structure.expand(&ident, &generics, runtime),
            Body::Enum(variants) => Body::expand_enum(&ident, &generics, &variants, runtime),
        }
    }
}

/// Report policy shared by a struct or selected by an enum's active alternative.
#[derive(Debug)]
enum Body {
    /// A struct whose fields share a diagnostic policy.
    Struct(
        /// Reporting contract with field selectors resolved against the struct.
        Box<Structure>,
    ),

    /// Alternatives sharing string messages and `Simple::Error` classification.
    Enum(
        /// Contracts for wording and source targets belonging to the declared alternatives.
        Vec<Variant>,
    ),
}

impl Body {
    /// Preserve each alternative's wording and target within an enum that acts
    /// as its own annotation and always has `Simple::Error` classification.
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

/// A struct's report policy with unambiguous targets and compatible metadata.
/// Its field types must still satisfy the runtime traits required by that policy.
#[derive(Debug)]
// NOTE(invariant): Policy In CompatibleReportPolicies(ResolvedMembers)
struct Structure {
    /// Contract for diagnostic wording, source location, and annotation collection selection.
    policy: StructurePolicy,
}

impl Structure {
    /// A struct must select forwarding, stored annotations, or an unambiguous primary target.
    fn new(attributes: Vec<Attribute>, fields: Fields, ident: &Ident) -> syn::Result<Self> {
        let members = Self::members(fields);
        let policy = StructurePolicy::parse(&attributes, &members, ident)?;

        Ok(Self { policy })
    }

    /// Declared field identities and types, preserving named selectors or tuple positions.
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

    /// Only a type path ending in `Span` qualifies for direct span targeting.
    /// Aliases and underlying type identity are unavailable to this spelling check.
    fn is_span(ty: &Type) -> bool {
        match ty {
            Type::Path(path) => path.path.segments.last().map(|segment| segment.ident == "Span").unwrap_or(false),
            _ => false,
        }
    }

    /// Require an explicit selector to name a declared field, retaining the type
    /// needed by that field's reporting contract.
    fn member(members: &[(Member, Type)], selected: &Member, ident: &Ident) -> syn::Result<TargetMember> {
        members
            .iter()
            .find(|(member, _)| member == selected)
            .map(|(member, ty)| TargetMember::new(member.clone(), ty.clone()))
            .ok_or_else(|| syn::Error::new_spanned(ident, "report member does not exist"))
    }

    /// Require exactly one syntactically named `Span`, or a sole field whose
    /// annotation contract can supply the target when no `Span` is present.
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

    /// Report implementations retain declared bounds rather than inferring requirements from fields.
    fn expand(&self, ident: &Ident, generics: &syn::Generics, runtime: &TokenStream) -> TokenStream {
        let Self { policy } = self;

        policy.expand(ident, generics, runtime)
    }
}

/// Defines a struct's report through forwarding, a selected source target, or
/// stored annotations. Each mode determines both the enclosing annotation's view
/// and the report's heading, classification, and annotation collection.
#[derive(Debug)]
enum StructurePolicy {
    /// Preserve a wrapped field's annotation and report contracts, including associated types.
    Transparent(
        /// Field supplying the message, target, heading, classification, and annotations.
        TargetMember,
    ),

    /// The enclosing struct is its own annotation with custom wording and an inferred target.
    Custom {
        /// Heading text independent of the primary annotation message.
        title: TitlePolicy,

        /// Text describing the concern at the selected source location.
        message: Message,

        /// Direct span or annotated field establishing the primary source coordinates.
        target: TargetMember,
    },

    /// Expose stored annotations while giving the enclosing struct its primary annotation's target.
    Stored {
        /// Heading text independent of the stored annotation messages.
        title: TitlePolicy,

        /// Optional wording for the enclosing annotation view, leaving the stored collection unchanged.
        message: Option<Message>,

        /// Collection whose `PrimaryAnnotations` contract establishes a primary location and message.
        annotations: TargetMember,
    },
}

impl StructurePolicy {
    /// Admit forwarding alone or a titled report with inferred or stored annotations,
    /// rejecting absent fields and incompatible wording choices.
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

    /// The selected mode determines the report's heading and collection together
    /// with the enclosing value's annotation contract.
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

    /// The enclosing struct supplies its own string annotation and heading with
    /// the selected target and `Simple::Error` classification.
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

    /// Preserve the selected field's message and report types, classification,
    /// and annotation collection in the enclosing struct.
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

    /// Expose the stored collection unchanged and use its primary target for the
    /// enclosing annotation, with an optional override as `str` wording.
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

/// Report options before compatibility and field selector validation.
/// Forwarding distinguishes an explicit field from inference of the sole field.
#[derive(Debug, Default)]
// NOTE(invariant): None.
struct Declarations {
    /// Forwarding selection, where `Some(None)` requests the sole field and `None` disables forwarding.
    transparent: Option<Option<Member>>,

    /// Literal report heading, mutually exclusive with `title_with`.
    title: Option<LitStr>,

    /// Projection borrowing the report heading, mutually exclusive with `title`.
    title_with: Option<Path>,

    /// Literal annotation wording, mutually exclusive with `message_with`.
    message: Option<LitStr>,

    /// Borrowed annotation wording projected from the report value or variant payload.
    message_with: Option<Path>,

    /// Explicit collection selector that replaces inferred primary targeting for structs.
    annotations: Option<Member>,
}

impl Declarations {
    /// Admit options across attributes with unique recognized names, without
    /// establishing mode compatibility or field existence.
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

/// Non-forwarding report options whose duplicates and unknown names are rejected.
/// Their compatibility depends on whether the declaration is a struct or enum alternative.
#[derive(Debug, Default, FromMeta)]
// NOTE(invariant): None.
struct StandardDeclarations {
    /// Literal heading independent of error display and annotation wording.
    title: Option<LitStr>,

    /// Borrowed heading projection available only to struct report policies.
    title_with: Option<Path>,

    /// Literal wording at the primary location, independent of the report heading.
    message: Option<LitStr>,

    /// Borrowed annotation wording projected from a struct or an enum payload.
    message_with: Option<Path>,

    /// Struct field whose collection supplies both annotation order and a primary location.
    annotations: Option<MemberArgument>,
}

/// A stored annotation field selector using a field name or tuple position.
/// Selecting an existing field is a separate obligation of the struct policy.
#[derive(Debug)]
// NOTE(invariant): None.
struct MemberArgument(Member);

impl FromMeta for MemberArgument {
    fn from_expr(value: &syn::Expr) -> darling::Result<Self> {
        syn::parse2(value.to_token_stream()).map(Self).map_err(darling::Error::custom)
    }
}

/// A report heading independent of annotation messages and standard error display.
#[derive(Debug)]
enum TitlePolicy {
    /// Literal heading used without format interpolation.
    Static(
        /// Heading text shared by every report value using this policy.
        LitStr,
    ),

    /// A heading borrowed from the diagnostic value.
    With(
        /// Projection whose result must coerce to `&str` for the subject borrow.
        Path,
    ),
}

impl TitlePolicy {
    /// Require exactly one heading source, rejecting both absent and competing declarations.
    fn resolve(title: Option<LitStr>, title_with: Option<Path>, ident: &Ident) -> syn::Result<Self> {
        match (title, title_with) {
            (Some(title), None) => Ok(Self::Static(title)),
            (None, Some(path)) => Ok(Self::With(path)),
            (None, None) => Err(syn::Error::new_spanned(ident, "Report requires title or title_with metadata")),
            (Some(_), Some(_)) => Err(syn::Error::new_spanned(ident, "title and title_with are mutually exclusive")),
        }
    }

    /// A heading expression valid for the borrow of `subject`, independent of annotation wording.
    fn expression(&self, subject: TokenStream) -> TokenStream {
        match self {
            Self::Static(title) => quote!(#title),
            Self::With(path) => quote!(#path(#subject)),
        }
    }
}

/// A selected field with its declared type retained for report requirements.
/// Only a type path ending in `Span` permits direct coordinate access.
#[derive(Debug, Clone)]
enum TargetMember {
    /// A field spelled as a `Span` supplies source coordinates directly.
    Span(
        /// Field selector locating the direct source coordinates.
        Member,
        /// Caller-declared type retained for associated-type references.
        Type,
    ),

    /// A field subject to the selected policy's runtime traits rather than direct span access.
    Annotated(
        /// Field selector available for annotation, report forwarding, or stored collection access.
        Member,
        /// Caller-declared type retained for associated-type references.
        Type,
    ),
}

impl TargetMember {
    /// Only `Span` spelling permits direct coordinate access, leaving other fields
    /// subject to the runtime traits required by their selected policy.
    fn new(member: Member, ty: Type) -> Self {
        if Structure::is_span(&ty) {
            Self::Span(member, ty)
        } else {
            Self::Annotated(member, ty)
        }
    }

    /// The selected field remains borrowed from the enclosing report for forwarding
    /// or stored annotation access.
    fn reference(&self) -> TokenStream {
        let member = match self {
            Self::Span(member, _) | Self::Annotated(member, _) => member,
        };

        quote!(&self.#member)
    }

    /// Declared field type governing the associated types of forwarded or stored diagnostics.
    const fn ty(&self) -> &Type {
        match self {
            Self::Span(_, ty) | Self::Annotated(_, ty) => ty,
        }
    }

    /// Source coordinates supplied by the field or its annotation contract,
    /// without validation against a source.
    fn expression(&self, runtime: &TokenStream) -> TokenStream {
        match self {
            Self::Span(member, _) => quote!(self.#member),
            Self::Annotated(member, _) => quote!(#runtime::annotated::Annotated::target(&self.#member)),
        }
    }
}

/// Annotation wording independent of the diagnostic heading and error display.
/// A projection borrows its subject rather than requiring allocated message text.
#[derive(Debug)]
enum Message {
    /// Literal annotation wording used without format interpolation.
    Static(
        /// Annotation text shared by every diagnostic value using this policy.
        LitStr,
    ),

    /// Annotation wording borrowed from a struct or enum payload.
    With(
        /// Projection whose result must coerce to `&str` for the subject borrow.
        Path,
    ),
}

impl Message {
    /// Permit absent wording for a stored primary message, rejecting competing overrides.
    fn optional(message: Option<LitStr>, message_with: Option<Path>, ident: &Ident) -> syn::Result<Option<Self>> {
        match (message, message_with) {
            (Some(message), None) => Ok(Some(Self::Static(message))),
            (None, Some(path)) => Ok(Some(Self::With(path))),
            (None, None) => Ok(None),
            (Some(_), Some(_)) => Err(syn::Error::new_spanned(ident, "message and message_with are mutually exclusive")),
        }
    }

    /// Require exactly one message source, rejecting both absent and competing declarations.
    fn resolve(message: Option<LitStr>, message_with: Option<Path>, ident: &Ident) -> syn::Result<Self> {
        match (message, message_with) {
            (Some(message), None) => Ok(Self::Static(message)),
            (None, Some(path)) => Ok(Self::With(path)),
            (None, None) => Err(syn::Error::new_spanned(ident, "Report requires message or message_with metadata")),
            (Some(_), Some(_)) => Err(syn::Error::new_spanned(ident, "message and message_with are mutually exclusive")),
        }
    }

    /// A message expression valid for the borrow of `subject`, independent of the report heading.
    fn expression(&self, subject: TokenStream) -> TokenStream {
        match self {
            Self::Static(message) => quote!(#message),
            Self::With(path) => quote!(#path(#subject)),
        }
    }
}

/// A report alternative with exactly one unnamed field and compatible wording
/// metadata. Its declared payload must satisfy the selected runtime contracts.
#[derive(Debug)]
// NOTE(invariant): UnnamedFields(Ident) = [Ty] And Policy In ValidReportPolicies(Ty)
struct Variant {
    /// Alternative whose wording and location apply when it is active.
    ident: Ident,

    /// Payload type whose spelling distinguishes a direct span from an annotated target.
    ty: Type,

    /// Explicit or forwarded wording within the enum's shared annotation and error kind.
    policy: Policy,

    /// Conditions under which this alternative participates in the enum's report contracts.
    cfg: Vec<Attribute>,
}

impl Variant {
    /// Require a sole unnamed payload and wording compatible with enum report constraints.
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

    /// This alternative's annotation wording must coerce to `&str` for the payload borrow.
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

    /// This alternative's source coordinates come from a direct span or the payload's annotation.
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

    /// This alternative's report heading is literal text or borrowed from its wrapped report.
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

/// An enum alternative's wording within a shared `str` annotation and `Simple::Error`
/// classification. Forwarding reuses wording rather than the wrapped report's collection or kind.
#[derive(Debug)]
enum Policy {
    /// Borrow annotation wording and the heading from a payload implementing both diagnostic traits.
    Transparent,

    /// Require a literal heading with independently selected annotation wording.
    Custom {
        /// Literal report heading independent of the payload's own diagnostic title.
        title: LitStr,

        /// Literal wording or text borrowed from the payload for the enum's annotation view.
        message: Message,
    },
}

impl Policy {
    /// Require transparent wording or a literal heading with a message, excluding
    /// struct-only title projections, stored collections, and field selectors.
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
