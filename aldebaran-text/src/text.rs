//! Stateful traversal and matching over borrowed source text.
//!
//! [`Text`] owns a source boundary and advances it through component-oriented
//! consumption, assertions, and lookahead. Branches support speculative work
//! without committing the parent cursor, while lexing builds higher-level values
//! from the same source-relative coordinates.

pub mod assert;

pub mod error;

pub mod consume;

pub mod internment;

pub mod stream;

pub mod branch;

use core::{
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
};

use aldebaran_logic::prelude::{AsAssert, Assert, Choose, Input};

use aldebaran_source::prelude::{Component, SourceDissect, SourceIter};

use aldebaran_span::prelude::Span;

use self::{branch::Branch, error::Expected};

/// A source cursor used for incremental matching and lexical consumption.
///
/// The cursor retains the original borrowed source and one absolute boundary.
/// Consumption advances only through source components, while lookahead and
/// branching allow callers to inspect or speculate without inventing separate
/// coordinate state.
pub struct Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// The current index in the source code.
    source_index: usize,

    /// The source code, borrowed for `'source`.
    source_code: &'source S,
}

impl<'source, S> Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Create a new [`Text`] from the specified [`Source `S`.
    #[inline]
    pub const fn create(source_code: &'source S) -> Self {
        Self::at(source_code, 0)
    }

    /// Create text positioned at one source boundary.
    #[inline]
    #[must_use]
    pub const fn at(source_code: &'source S, source_index: usize) -> Self {
        Self { source_index, source_code }
    }
}

impl<'source, S> Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Retrieve the index of this [`Text`] respective to the source code.
    #[inline]
    pub const fn index(&self) -> usize {
        let &Self { source_index, .. } = self;

        source_index
    }

    /// Retrieve the source code of this [`Text`].
    #[inline]
    pub const fn source(&self) -> &'source S {
        let &Self { source_code, .. } = self;

        source_code
    }
}

impl<'source, S> Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Dissect this [`Text`] as per the given [`Span`].
    ///
    /// This abides by the same rules as [`SourceDissect::dissect`].
    #[inline]
    pub fn slice(&self, span: Span) -> Text<'source, S>
    where
        S: SourceDissect<'source, View<'source> = &'source S>,
    {
        let &Self { source_code, .. } = self;
        let source_code: &'source S = source_code.dissect(span);
        let source_index = span.start();

        Text { source_index, source_code }
    }

    /// Mutably borrow this [`Text`], creating a new [`Branch`] from it.
    ///
    /// Branching allows for arbitrary speculative operations to be performed on
    /// a [`Text`], and commiting the changes if necessary.
    #[inline]
    pub fn branch<'borrow>(&'borrow mut self) -> Branch<'source, 'borrow, S>
    where
        'source: 'borrow,
    {
        Branch::from(self)
    }
}

impl<'source, S> Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Determine if the end of the source text has been reached.
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.peek().is_none()
    }

    /// Peek at the immediate [`Component`] from the source text.
    #[inline]
    pub fn peek(&self) -> Option<S::Component> {
        self.peek_at::<0>()
    }

    /// Peek at the `n`th immediate [`Component`] from the source text.
    ///
    /// `n` is zero-indexed, so `peek_at::<0>()` is equivalent to [`Self::peek`].
    #[inline]
    pub fn peek_at<const N: usize>(&self) -> Option<S::Component> {
        let &Self { source_index, source_code } = self;

        source_code.offset(source_index)?.iter().nth(N)
    }

    /// Take the next [`Component`] from the source text, irrevocably.
    #[inline]
    pub fn any(&mut self) -> Option<S::Component> {
        let &mut Self {
            ref mut source_index,
            source_code,
        } = self;

        let target_input = source_code.offset(*source_index)?.iter().next()?;

        *source_index += target_input.size().get();

        Some(target_input)
    }

    /// Take the next [`Component`] from the source text. Includes its associated [`Span`].
    #[inline]
    pub fn any_spanned(&mut self) -> Option<(S::Component, Span)> {
        let &mut Self {
            ref mut source_index,
            source_code,
        } = self;
        let target_input = source_code.offset(*source_index)?.iter().next()?;

        let span = Span::new(*source_index, target_input.size());

        *source_index += target_input.size().get();

        Some((target_input, span))
    }

    /// Skip the next [`Component`] from the source text.
    #[inline]
    pub fn skip(&mut self) {
        let _ = self.any();
    }
}

impl<'source, S> Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Irrevocably consumes the next component, determining if it is subject
    /// to the specified [`Assert`], bailing with an [`Expected`] otherwise.
    #[inline]
    pub fn expect_is_next<A, T>(&mut self, target_assert: T) -> Result<S::Component, Expected<S::Component, A>>
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component> + Clone,
        S::Component: Input,
    {
        self.expect_is_next_spanned(target_assert).map(|(component, _)| component)
    }

    /// Consume the next component and return its classification value.
    #[inline]
    pub fn expected_is_next<A>(&mut self, target_assert: A) -> Result<A::Unit, Expected<S::Component, A>>
    where
        A: Choose<S::Component> + Clone,
        S::Component: Input,
    {
        self.expected_is_next_spanned(target_assert).map(|(unit, _)| unit)
    }

    /// Alike to [`Self::is_next`], but reports an [`Expected`] instead of a
    /// boolean.
    #[inline]
    pub fn validate_is_next<A, T>(&self, target_assert: T) -> Result<S::Component, Expected<S::Component, A>>
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component> + Clone,
        S::Component: Input,
    {
        let &Self {
            source_index,
            source_code: _,
            ..
        } = self;

        let target_assert = target_assert.as_assert();

        let target_value = self.peek();

        let target_span = Span::unit(source_index);

        match target_value {
            Some(target_input) if target_assert.assert(target_input) => Ok(target_input),
            Some(target_exist) => {
                let target_assert = target_assert.clone();

                Err(Expected::found(target_assert, target_exist, target_span))
            }
            None => {
                let target_assert = target_assert.clone();

                Err(Expected::this(target_assert, Span::unit(source_index)))
            }
        }
    }

    /// Validates that the next component is subject to the specified
    /// [`Assert`] or that `eof` has been reached, failing with an [`Expected`]
    /// otherwise.
    ///
    /// This won't consume the next component in case of failure.
    #[inline]
    pub fn validate_is_next_or_eof<A, T>(&self, target_assert: T) -> Result<(), Expected<S::Component, A>>
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component> + Clone,
        S::Component: Input,
    {
        let &Self {
            source_code: _,
            source_index,
            ..
        } = self;

        let target_assert = target_assert.as_assert();

        let target_start = source_index;

        let target_value = self.peek();

        let span = Span::unit(target_start);

        match target_value {
            Some(target_input) if target_assert.assert(target_input) => Ok(()),
            Some(target_exist) => {
                let target_assert = target_assert.clone();

                Err(Expected::found(target_assert, target_exist, span))
            }

            None => Ok(()),
        }
    }

    /// Optionally consumes the next component if subject to the specified
    /// [`Assert`], returning the value if it is.
    #[inline]
    pub fn optional<A, T>(&mut self, target_assert: T) -> Option<S::Component>
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component>,
        S::Component: Input,
    {
        if self.is_next_or_eof(target_assert) { self.any() } else { None }
    }
}

impl<'source, S> Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Expect the next [`Component`] to be subject to `T`. Includes its associated [`Span`].
    #[inline]
    pub fn expect_is_next_spanned<A, T>(&mut self, target_assert: T) -> Result<(S::Component, Span), Expected<S::Component, A>>
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component> + Clone,
        S::Component: Input,
    {
        let target_assert = target_assert.as_assert();

        match self.any_spanned() {
            Some((target_input, span)) if target_assert.assert(target_input) => Ok((target_input, span)),
            Some((target_exist, span)) => Err(Expected::found(target_assert.clone(), target_exist, span)),
            None => Err(Expected::this(target_assert.clone(), Span::unit(self.index()))),
        }
    }

    /// Consume the next component and return its classification with the source extent.
    #[inline]
    pub fn expected_is_next_spanned<A>(&mut self, target_assert: A) -> Result<(A::Unit, Span), Expected<S::Component, A>>
    where
        A: Choose<S::Component> + Clone,
        S::Component: Input,
    {
        match self.any_spanned() {
            Some((component, span)) => match target_assert.choose(component) {
                Some(unit) => Ok((unit, span)),
                None => Err(Expected::found(target_assert, component, span)),
            },
            None => Err(Expected::this(target_assert, Span::unit(self.index()))),
        }
    }

    /// Take the next [`Component`] if it is subject to `T`. Includes its associated [`Span`].
    #[inline]
    pub fn optional_spanned<A, T>(&mut self, target_assert: T) -> Option<(S::Component, Span)>
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component>,
        S::Component: Input,
    {
        if self.is_next_or_eof(target_assert) {
            self.any_spanned()
        } else {
            None
        }
    }

    /// Repeatedly ignores any components that satisfy the target [`Assert`]. Returns the [`Span`] of all ignored components.
    #[inline]
    pub fn ignore_while_spanned<A, T>(&mut self, target_assert: T) -> Option<Span>
    where
        A: Assert<S::Component>,
        for<'b> &'b T: AsAssert<S::Component, A>,
        S::Component: Input,
    {
        let mut span = None::<Span>;

        while let Some((.., target_span)) = self.optional_spanned(&target_assert) {
            match span {
                Some(ref mut acc_span) => *acc_span = acc_span.superset(target_span),
                None => span = Some(target_span),
            }
        }

        span
    }
}

impl<'source, S> Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Determine if the next component is subject to the specified [`Assert`], falling back to the default value if the end of the source
    /// has been reached.
    #[inline]
    fn is_next_or_default<const D: bool, A, T>(&self, target_assert: T) -> bool
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component>,
        S::Component: Input,
    {
        let target_assert = target_assert.as_assert();

        self.peek().map_or(D, |target_input| target_assert.assert(target_input))
    }

    /// Determine whether the component at lookahead `N` satisfies the specified [`Assert`].
    ///
    /// Lookahead is zero-indexed, so `is_at::<0>(assertion)` examines the same
    /// component as [`Self::is_next`].
    #[inline]
    pub fn is_at<const N: usize, A, T>(&self, target_assert: T) -> bool
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component>,
        S::Component: Input,
    {
        let target_assert = target_assert.as_assert();

        self.peek_at::<N>()
            .map(|target_input| target_assert.assert(target_input))
            .unwrap_or(false)
    }

    /// Determines if the next component is subject to the specified [`Assert`].
    #[inline]
    pub fn is_next<A, T>(&self, target_assert: T) -> bool
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component>,
        S::Component: Input,
    {
        self.is_at::<0, A, T>(target_assert)
    }

    /// Determine whether the immediate component satisfies the target [`Assert`] or if the end of the source has been reached.
    #[inline]
    pub fn is_next_or_eof<A, T>(&self, target_assert: T) -> bool
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component>,
        S::Component: Input,
    {
        self.is_next_or_default::<true, A, T>(target_assert)
    }

    /// Repeatedly ignores any components that satisfy the target [`Assert`].
    #[inline]
    pub fn ignore_while<A, T>(&mut self, target_assert: T)
    where
        A: Assert<S::Component>,
        for<'b> &'b T: AsAssert<S::Component, A>,
        S::Component: Input,
    {
        while let Some(_) = self.optional(&target_assert) {}
    }

    /// Counts the number of components that satisfy the target [`Assert`].
    #[inline]
    pub fn count<A, T>(&mut self, target_assert: T) -> usize
    where
        A: Assert<S::Component>,
        for<'b> &'b T: AsAssert<S::Component, A>,
        S::Component: Input,
    {
        let mut target_count = usize::MIN;

        while self.is_next(&target_assert) {
            let _ = self.any();

            target_count += 1;
        }

        target_count
    }
}

impl<'source, S> Deref for Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    type Target = S;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let &Self { source_code, .. } = self;

        source_code
    }
}

impl<'source, S> Default for Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    for<'a> &'a S: Default,
{
    #[inline]
    fn default() -> Self {
        let source_code = Default::default();
        let source_index = usize::MIN;

        Self { source_index, source_code }
    }
}

impl<'source, S: Hash> Hash for Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        let &Self { source_index, source_code } = self;

        source_index.hash(state);
        source_code.hash(state);
    }
}

impl<'source, S> Eq for Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    S: Eq,
{
}

impl<'source, S> PartialEq for Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    S: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.source_index == other.source_index && self.source_code == other.source_code
    }
}

impl<'source, S> Copy for Text<'source, S> where S: SourceDissect<'source> + SourceIter<'source> + ?Sized {}

impl<'source, S> Clone for Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        let &Self { source_index, source_code } = self;

        Self { source_index, source_code }
    }
}

impl<'source, S> fmt::Debug for Text<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Text")
            .field("source_index", &self.source_index)
            .field("source_code", &self.source_code)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use aldebaran_logic::prelude::Connective as _;
    use aldebaran_report::{
        annotated::Annotated,
        render::{
            RenderMut,
            textual::{
                Present, Textual,
                offload::fancy::{Fancy, FancySettings, Fancyness},
            },
        },
        report::Report as _,
    };

    use aldebaran_span::span::Span;

    use crate::prelude::{AsciiAlphanumeric, Digit};

    use super::Text;

    #[test]
    fn missing_input_targets_terminal_source_boundary() {
        let source = "a";
        let mut text = Text::create(source);

        assert_eq!(text.any(), Some('a'));

        let error = text.expect_is_next(AsciiAlphanumeric).expect_err("source should be exhausted");
        let target = Annotated::target(&error);

        assert_eq!(target.start(), source.len());
        assert_eq!(target.length(), core::num::NonZero::<usize>::MIN);
        assert_eq!(target.end(), source.len() + 1);
    }

    #[test]
    fn lookahead_counts_components_and_preserves_width() {
        let mut text = Text::create("αb");

        assert_eq!(text.peek_at::<0>(), Some('α'));
        assert_eq!(text.peek_at::<1>(), Some('b'));
        assert_eq!(
            text.any_spanned(),
            Some(('α', Span::new(0, core::num::NonZero::new(2).expect("width"))))
        );
        assert_eq!(text.index(), 2);
        assert_eq!(text.peek(), Some('b'));
    }

    #[test]
    fn a() {
        let s = &"bb"[..];

        let mut t = Text::create(s);

        let err = t.expect_is_next(AsciiAlphanumeric.or('_').and(Digit)).expect_err("valid");

        dbg!(err);

        let err = t.expect_is_next(AsciiAlphanumeric.or('_').and(Digit)).expect_err("valid");

        dbg!(&err);

        //let err = t.expect_is_next(Char('8')).expect_err("valid");

        let mut target_sink = String::new();

        let report = err.attach(s);
        let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut target_sink);

        renderer
            .render_mut_with_input(Present::from_input(FancySettings::level(Fancyness::Outblown)), &report)
            .expect("failed to render error");

        println!("{}", target_sink);
    }
}
