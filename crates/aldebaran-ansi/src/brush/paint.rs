//! Deferred styling for printable values.

use core::fmt;

use aldebaran_print::{prelude::Print, style::Stylus};

use crate::prelude::{Color, ContextualPredicate, Style};

use super::style::TextAttribute;

/// A printable value with deferred ANSI styling.
///
/// Printing applies the stored [`Style`] around the subject's output when
/// styling is enabled, but delegates directly to the subject when it is disabled.
/// Color settings replace earlier choices for the same surface while text
/// attributes accumulate, so the full style can be built before output begins.
/// Enabled output inherits [`Style`]'s global reset behavior and
/// error precedence, including reset failure after subject output. Nested
/// painting does not restore an enclosing style after an inner reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// NOTE(invariant): None.
pub struct Painted<T>
where
    T: Paintable,
{
    /// The printable value receiving deferred styling.
    subject: T,

    /// The terminal settings emitted around the subject when enabled.
    style: Style,

    /// Whether printing applies the stored style.
    enabled: bool,
}

impl<T> Painted<T>
where
    T: Paintable,
{
    /// Snapshot a contextual presentation decision before rendering.
    ///
    /// The predicate is evaluated immediately, replacing the enable state. The
    /// predicate and its context are not retained for later printing.
    #[inline]
    pub fn only_when<U>(mut self, target_predicate: &U) -> Self
    where
        U: ContextualPredicate,
    {
        let Self { ref mut enabled, .. } = self;

        *enabled = target_predicate.holds();

        self
    }

    /// Set whether printing applies the stored style.
    #[inline]
    pub fn enabled(mut self, target_state: bool) -> Self {
        let Self { ref mut enabled, .. } = self;

        *enabled = target_state;

        self
    }

    /// Replace all stored style settings without changing the enable state.
    #[inline]
    pub fn styled(mut self, target_style: Style) -> Self {
        let Self { ref mut style, .. } = self;

        *style = target_style;

        self
    }
}

impl<T> Print for Painted<T>
where
    T: Paintable,
    T: Print,
{
    type Context = T::Context;

    #[inline]
    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self {
            ref subject,
            ref style,
            enabled,
        } = self;

        if enabled {
            match style.style(writer, |writer| subject.print_with_context(writer, context)) {
                Ok(target_value) => target_value,
                Err(target_error) => Err(target_error),
            }
        } else {
            subject.print_with_context(writer, context)
        }
    }
}

/// A macro that normalizes the given items to be trait-definition-compatible.
///
/// For example, for a function item, it will strip visibility and constness
/// modifiers. Other modifiers such as unsafeness are kept intact.
macro_rules! Normalize {
    () => {};
    (@attribute [$($target_kept:tt)*] [] [$($target_item:tt)*]) => {
        $($target_kept)*
        $($target_item)*
    };
    (@attribute [$($target_kept:tt)*] [#[inline $($target_args:tt)*] $($target_rest:tt)*] [$($target_item:tt)*]) => {
        Normalize!(@attribute [$($target_kept)*] [$($target_rest)*] [$($target_item)*]);
    };
    (@attribute [$($target_kept:tt)*] [#[$($target_meta:tt)*] $($target_rest:tt)*] [$($target_item:tt)*]) => {
        Normalize!(@attribute [$($target_kept)* #[$($target_meta)*]] [$($target_rest)*] [$($target_item)*]);
    };
    // Base case.
    (return become for $target_trait_ident:ident use $target_type_ident:ident:) => {};
    (
        return become for $target_trait_ident:ident use $target_type_ident:ident:

        $(
            #[$($target_meta:tt)*]
        )*
        $target_vis:vis const fn $target_name:ident(
            $(
                $target_arg_tt:tt
            )*
        ) $(-> $target_ret:ty)? $target_body:block

        $($target_rest:tt)*
    ) => {
        Normalize!(@attribute [] [$(#[$($target_meta)*])*] [
        // Strip visibility, constness, the body, and implementation-only inline attributes.
        fn $target_name(
            $(
                $target_arg_tt
            )*
        ) $(-> <$target_type_ident as $target_trait_ident>::Parametrized<$target_ret>)? where Self: Sized;
        ]);

        Normalize!(return become for $target_trait_ident use $target_type_ident: $($target_rest)*);
    };
    (
        return become for $target_trait_ident:ident use $target_type_ident:ident:

        $(
            #[$($target_meta:tt)*]
        )*
        $target_vis:vis const unsafe fn $target_name:ident(
            $(
                $target_arg_tt:tt
            )*
        ) $(-> $target_ret:ty)? $target_body:block

        $($target_rest:tt)*
    ) => {
        Normalize!(@attribute [] [$(#[$($target_meta)*])*] [
        // Preserve unsafety while stripping implementation-only syntax and attributes.
        unsafe fn $target_name(
            $(
                $target_arg_tt
            )*
        ) $(-> <$target_type_ident as $target_trait_ident>::Parametrized<$target_ret>)? where Self: Sized;
        ]);

        Normalize!(return become for $target_trait_ident use $target_type_ident: $($target_rest)*);
    };
    (
        return become for $target_trait_ident:ident use $target_type_ident:ident:

        $(
            #[$($target_meta:tt)*]
        )*
        $target_vis:vis unsafe fn $target_name:ident(
            $(
                $target_arg_tt:tt
            )*
        ) $(-> $target_ret:ty)? $target_body:block

        $($target_rest:tt)*
    ) => {
        Normalize!(@attribute [] [$(#[$($target_meta)*])*] [
        // Preserve unsafety while stripping implementation-only syntax and attributes.
        unsafe fn $target_name(
            $(
                $target_arg_tt
            )*
        ) $(-> <$target_type_ident as $target_trait_ident>::Parametrized<$target_ret>)? where Self: Sized;
        ]);

        Normalize!(return become for $target_trait_ident use $target_type_ident: $($target_rest)*);
    };
    (
        return become for $target_trait_ident:ident use $target_type_ident:ident:

        $(
            #[$($target_meta:tt)*]
        )*
        $target_vis:vis fn $target_name:ident(
            $(
                $target_arg_tt:tt
            )*
        ) $(-> $target_ret:ty)? $target_body:block

        $($target_rest:tt)*
    ) => {
        Normalize!(@attribute [] [$(#[$($target_meta)*])*] [
        // Strip visibility, the body, and implementation-only inline attributes.
        fn $target_name(
            $(
                $target_arg_tt
            )*
        ) $(-> <$target_type_ident as $target_trait_ident>::Parametrized<$target_ret>)? where Self: Sized;
        ]);

        Normalize!(return become for $target_trait_ident use $target_type_ident: $($target_rest)*);
    };
}

/// Serves as a macro to automatically relay the methods of a type to a trait.
///
/// This works in a way that the output types of the methods of the trait are
/// parametrized over the bridged type. An example of this is a method in the
/// concrete type `Concrete` returns `Foo`, but in the trait `Concretized` it
/// returns `<Concrete as Bridged>::Parametrized<Foo>`.
/// Note that the `Bridged` trait is nonexistent, and is only used to convey
/// the idea. Instead, an anonymous trait is created for each type that is
/// bridged due to the, as of writing, unstable status of inherent associated
/// types.
///
/// See [`Paintable`] for further information.
macro_rules! Bridge {
    () => {};
    (
        $(
            #[$target_meta:meta]
        )*

        use $target_bridge_trait:ident impl $target_type_ident:ident for trait $target_trait_vis:vis $target_trait:ident $(where [$($target_trait_where:tt)*])? {
            $(
                $target_item_tt:tt
            )*
        }

        type $(
            <
                $(
                    $target_parametrize_tt:tt
                ),*
            >
        )? = $target_parametrize_type:ty
        $(where [$($target_parametrize_where:tt)*])?;
    ) => {
        $(
            #[$target_meta]
        )*
        $target_trait_vis trait $target_trait
        $(where $($target_trait_where)*)?
        {
            // For each method, convert it to one suitable for the trait.
            //
            // This means removing any visibility and constness modifiers.
            Normalize!(
                return become for $target_bridge_trait use $target_type_ident:

                $(
                    $target_item_tt
                )*
            );
        }
    };
    (
        $(
            #[$target_meta:meta]
        )*

        impl
            $(
                <
                    $(
                        $target_generic_tt:tt
                    ),*
                >
            )?
        for $target_type:ty $(where [$($target_where:tt)*])?: become $target_trait_vis:vis $target_trait:ident $(where [$($target_trait_where:tt)*])? {
            $(
                $target_item_tt:tt
            )*
        }

        type $(
            <
                $(
                    $target_parametrize_tt:tt
                ),*
            >
        )? = $target_parametrize_type:ty
        $(where [$($target_parametrize_where:tt)*])?;

        become $target_callback_macro:ident;
    ) => {
        impl
            $(
                <
                    $(
                        $target_generic_tt
                    ),*
                >
            )?
        $target_type $(where $($target_where)*)? {
            $(
                $target_item_tt
            )*
        }

        tokel::stream!(
            /// An automatically generated trait that serves as a bridge between a
            /// concrete type and a trait.
            #[doc(hidden)]
            $target_trait_vis trait [< $target_trait Bridge >]:concatenate {
                /// Wrapper preserving the printable subject type across builder operations.
                type Parametrized $(
                    <
                        $(
                            $target_parametrize_tt
                        ),*
                    >
                )?
                $(where $($target_parametrize_where)*)?;
            }

            /// A dummy, uninhabited type that serves as an implementation detail.
            #[doc(hidden)]
            #[derive(Debug)]
            $target_trait_vis enum [< Ty $target_trait Bridge >]:concatenate {}

            #[doc(hidden)]
            #[automatically_derived]
            impl [< $target_trait Bridge >]:concatenate for [< Ty $target_trait Bridge >]:concatenate {
                /// Wrapper preserving the printable subject type across builder operations.
                type Parametrized $(
                    <
                        $(
                            $target_parametrize_tt
                        ),*
                    >
                )? = $target_parametrize_type
                $(where $($target_parametrize_where)*)?;
            }

            // Bridge the methods to the trait.
            Bridge!(
                $(
                    #[$target_meta]
                )*
                use [< $target_trait Bridge >]:concatenate impl [< Ty $target_trait Bridge >]:concatenate for trait $target_trait_vis $target_trait $(where [$($target_trait_where)*])? {
                    $(
                        $target_item_tt
                    )*
                }

                type $(
                    <
                        $(
                            $target_parametrize_tt
                        ),*
                    >
                )? = $target_parametrize_type
                $(where [$($target_parametrize_where)*])?;
            );



            $target_callback_macro!(
                $(
                    #[$target_meta]
                )*
                impl
                $(
                    <
                        $(
                            $target_generic_tt
                        ),*
                    >
                )?
                for $target_type $(where [$($target_where)*])?: become $target_trait_vis $target_trait $(where [$($target_trait_where)*])? {
                    $(
                        $target_item_tt
                    )*
                }
            );
        );
    };
}

/// Implementation detail for arbitrarily blanketing a type.
#[doc(hidden)]
trait Blanketable {
    /// The type to instantiate for.
    type For;

    /// The output for an instantiation operation.
    type Output;

    /// Instantiate this type for the given type.
    fn instantiate_for(_: Self::For) -> Self::Output;
}

/// Forward painting methods through the blanket instantiation boundary.
macro_rules! Blanket {
    () => {};
    (fn for $target_type:ty:) => {};
    (
        fn for $target_type:ty:

        $(
            #[$target_meta:meta]
        )*

        $target_vis:vis const fn $target_name:ident(
            $(
                $target_arg_tt:tt
            )*
        ) $(-> $target_ret:ty)? $target_body:block

        $($target_rest:tt)*
    ) => {
        $(
            #[$target_meta]
        )*
        fn $target_name(
            $(
                $target_arg_tt
            )*
        ) -> $target_type {
            <$target_type as Blanketable>::instantiate_for($($target_arg_tt)*).$target_name()
        }

        Blanket!(fn for $target_type: $($target_rest)*);
    };
    (
        fn for $target_type:ty:

        $(
            #[$target_meta:meta]
        )*

        $target_vis:vis unsafe fn $target_name:ident(
            $(
                $target_arg_tt:tt
            )*
        ) $(-> $target_ret:ty)? $target_body:block

        $($target_rest:tt)*
    ) => {
        $(
            #[$target_meta]
        )*
        unsafe fn $target_name(
            $(
                $target_arg_tt
            )*
        ) -> $target_type {
            <$target_type as Blanketable>::instantiate_for($($target_arg_tt)*).$target_name()
        }

        Blanket!(fn for $target_type: $($target_rest)*);
    };
    (
        fn for $target_type:ty:

        $(
            #[$target_meta:meta]
        )*

        $target_vis:vis fn $target_name:ident(
            $(
                $target_arg_tt:tt
            )*
        ) $(-> $target_ret:ty)? $target_body:block

        $($target_rest:tt)*
    ) => {
        $(
            #[$target_meta]
        )*
        fn $target_name(
            $(
                $target_arg_tt
            )*
        ) -> $target_type {
            <$target_type as Blanketable>::instantiate_for($($target_arg_tt)*).$target_name()
        }

        Blanket!(fn for $target_type: $($target_rest)*);
    };
    (
        $(
            #[$target_meta:meta]
        )*

        impl
            $(
                <
                    $(
                        $target_generic_tt:tt
                    ),*
                >
            )?
        for $target_type:ty $(where [$($target_where:tt)*])?: become $target_trait_vis:vis $target_trait:ident $(where [$($target_trait_where:tt)*])? {
            $(
                $target_item_tt:tt
            )*
        }
    ) => {
        impl <T> $target_trait for T $(where $($target_trait_where)*)? {
            Blanket!(
                fn for $target_type:

                $(
                    $target_item_tt
                )*
            );
        }
    };
}

#[doc(hidden)]
#[automatically_derived]
impl<T> Blanketable for Painted<T>
where
    T: Paintable,
{
    type For = T;

    type Output = Self;

    #[inline]
    fn instantiate_for(target_value: Self::For) -> Self::Output {
        Painted {
            subject: target_value,
            style: Style::default(),
            enabled: true,
        }
    }
}

Bridge!(
    /// Add color and text-attribute builders to a printable value.
    ///
    /// Color methods replace the value on their surface, while attribute
    /// methods accumulate. The resulting [`Painted`] value applies its style
    /// when printed.
    ///
    /// For custom settings, use [`Painted::styled`] with a [`Style`].
    impl<T> for Painted<T>
    where [T: Paintable]: become pub Paintable where [Self: Print] {
        /// Begin a style with no activation settings, or retain the current painting chain.
        /// Enabled printing still performs the style's final global reset.
        #[inline]
        pub const fn stylable(self) -> Self {
            self
        }

        /// Paint the subject with a `red` color.
        ///
        /// Results in the same effect as using the [`Color::RED`] constant.
        #[inline]
        pub const fn red(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::RED);

            target_value
        }

        /// Paint the subject with a `bright red` color.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_RED`] constant.
        #[inline]
        pub const fn bright_red(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::BRIGHT_RED);

            target_value
        }

        /// Paint the subject with a `green` color.
        ///
        /// Results in the same effect as using the [`Color::GREEN`] constant.
        #[inline]
        pub const fn green(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::GREEN);

            target_value
        }

        /// Paint the subject with a `bright green` color.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_GREEN`] constant.
        #[inline]
        pub const fn bright_green(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::BRIGHT_GREEN);

            target_value
        }

        /// Paint the subject with a `blue` color.
        ///
        /// Results in the same effect as using the [`Color::BLUE`] constant.
        pub const fn blue(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::BLUE);

            target_value
        }

        /// Paint the subject with a `bright blue` color.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_BLUE`] constant.
        pub const fn bright_blue(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::BRIGHT_BLUE);

            target_value
        }

        /// Paint the subject with a `yellow` color.
        ///
        /// Results in the same effect as using the [`Color::YELLOW`] constant.
        #[inline]
        pub const fn yellow(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::YELLOW);

            target_value
        }

        /// Paint the subject with a `bright yellow` color.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_YELLOW`] constant.
        #[inline]
        pub const fn bright_yellow(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::BRIGHT_YELLOW);

            target_value
        }

        /// Paint the subject with a `magenta` color.
        ///
        /// Results in the same effect as using the [`Color::MAGENTA`] constant.
        #[inline]
        pub const fn magenta(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::MAGENTA);

            target_value
        }

        /// Paint the subject with a `bright magenta` color.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_MAGENTA`] constant.
        #[inline]
        pub const fn bright_magenta(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::BRIGHT_MAGENTA);

            target_value
        }

        /// Paint the subject with a `cyan` color.
        ///
        /// Results in the same effect as using the [`Color::CYAN`] constant.
        #[inline]
        pub const fn cyan(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::CYAN);

            target_value
        }

        /// Paint the subject with a `bright cyan` color.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_CYAN`] constant.
        #[inline]
        pub const fn bright_cyan(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::BRIGHT_CYAN);

            target_value
        }

        /// Paint the subject with a `white` color.
        ///
        /// Results in the same effect as using the [`Color::WHITE`] constant.
        #[inline]
        pub const fn white(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::WHITE);

            target_value
        }

        /// Paint the subject with a `bright white` color.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_WHITE`] constant.
        #[inline]
        pub const fn bright_white(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::BRIGHT_WHITE);

            target_value
        }

        /// Paint the subject with a `black` color.
        ///
        /// Results in the same effect as using the [`Color::BLACK`] constant.
        #[inline]
        pub const fn black(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::BLACK);

            target_value
        }

        /// Paint the subject with a `bright black` color.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_BLACK`] constant.
        #[inline]
        pub const fn bright_black(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.foreground_replace(Color::BRIGHT_BLACK);

            target_value
        }

        /// Paint the subject with a `red` background.
        ///
        /// Results in the same effect as using the [`Color::RED`] constant.
        #[inline]
        pub const fn on_red(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::RED);

            target_value
        }

        /// Paint the subject with a `bright red` background.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_RED`] constant.
        #[inline]
        pub const fn on_bright_red(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::BRIGHT_RED);

            target_value
        }

        /// Paint the subject with a `green` background.
        ///
        /// Results in the same effect as using the [`Color::GREEN`] constant.
        #[inline]
        pub const fn on_green(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::GREEN);

            target_value
        }

        /// Paint the subject with a `bright green` background.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_GREEN`] constant.
        #[inline]
        pub const fn on_bright_green(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::BRIGHT_GREEN);

            target_value
        }

        /// Paint the subject with a `blue` background.
        ///
        /// Results in the same effect as using the [`Color::BLUE`] constant.
        #[inline]
        pub const fn on_blue(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::BLUE);

            target_value
        }

        /// Paint the subject with a `bright blue` background.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_BLUE`] constant.
        #[inline]
        pub const fn on_bright_blue(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::BRIGHT_BLUE);

            target_value
        }

        /// Paint the subject with a `yellow` background.
        ///
        /// Results in the same effect as using the [`Color::YELLOW`] constant.
        #[inline]
        pub const fn on_yellow(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::YELLOW);

            target_value
        }

        /// Paint the subject with a `bright yellow` background.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_YELLOW`] constant.
        #[inline]
        pub const fn on_bright_yellow(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::BRIGHT_YELLOW);

            target_value
        }

        /// Paint the subject with a `magenta` background.
        ///
        /// Results in the same effect as using the [`Color::MAGENTA`] constant.
        #[inline]
        pub const fn on_magenta(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::MAGENTA);

            target_value
        }

        /// Paint the subject with a `bright magenta` background.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_MAGENTA`] constant.
        #[inline]
        pub const fn on_bright_magenta(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::BRIGHT_MAGENTA);

            target_value
        }

        /// Paint the subject with a `cyan` background.
        ///
        /// Results in the same effect as using the [`Color::CYAN`] constant.
        #[inline]
        pub const fn on_cyan(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::CYAN);

            target_value
        }

        /// Paint the subject with a `bright cyan` background.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_CYAN`] constant.
        #[inline]
        pub const fn on_bright_cyan(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::BRIGHT_CYAN);

            target_value
        }

        /// Paint the subject with a `white` background.
        ///
        /// Results in the same effect as using the [`Color::WHITE`] constant.
        #[inline]
        pub const fn on_white(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::WHITE);

            target_value
        }

        /// Paint the subject with a `bright white` background.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_WHITE`] constant.
        #[inline]
        pub const fn on_bright_white(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::BRIGHT_WHITE);

            target_value
        }

        /// Paint the subject with a `black` background.
        ///
        /// Results in the same effect as using the [`Color::BLACK`] constant.
        #[inline]
        pub const fn on_black(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::BLACK);

            target_value
        }

        /// Paint the subject with a `bright black` background.
        ///
        /// Results in the same effect as using the [`Color::BRIGHT_BLACK`] constant.
        #[inline]
        pub const fn on_bright_black(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            let _: Option<Color> = style.background_replace(Color::BRIGHT_BLACK);

            target_value
        }

        /// Paint the subject with a `bold` style.
        ///
        /// This is equivalent to using the [`TextAttribute::Bold`] variant.
        #[inline]
        pub const fn bold(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            style.attribute_mut().force(TextAttribute::Bold);

            target_value
        }

        /// Paint the subject with a `dim` style.
        ///
        /// This is equivalent to using the [`TextAttribute::Dim`] variant.
        #[inline]
        pub const fn dim(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            style.attribute_mut().force(TextAttribute::Dim);

            target_value
        }

        /// Paint the subject with an `italic` style.
        ///
        /// This is equivalent to using the [`TextAttribute::Italic`] variant.
        #[inline]
        pub const fn italic(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            style.attribute_mut().force(TextAttribute::Italic);

            target_value
        }

        /// Paint the subject with an `underline` style.
        ///
        /// This is equivalent to using the [`TextAttribute::Underline`] variant.
        #[inline]
        pub const fn underline(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            style.attribute_mut().force(TextAttribute::Underline);

            target_value
        }

        /// Paint the subject with a `blink` style.
        ///
        /// This is equivalent to using the [`TextAttribute::Blink`] variant.
        #[inline]
        pub const fn blink(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            style.attribute_mut().force(TextAttribute::Blink);

            target_value
        }

        /// Paint the subject with a `inverse` style.
        ///
        /// This is equivalent to using the [`TextAttribute::Inverse`] variant.
        #[inline]
        pub const fn inverse(self) -> Self {
            let mut target_value = self;

            let &mut Self {
                ref mut style,
                ..
            } = &mut target_value;

            style.attribute_mut().force(TextAttribute::Inverse);

            target_value
        }
    }

    type<U> = Painted<U>
        where [U: Paintable];

    become Blanket;
);
