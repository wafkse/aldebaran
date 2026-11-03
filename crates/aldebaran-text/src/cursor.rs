//! Source traversal for lexical recognition and recovery.
//!
//! See [`Cursor`], [`branch::Branch`], and [`consume::Lex`] for further information.

pub mod error;

pub mod consume;

pub mod internment;

pub mod stream;

pub mod branch;

use core::{
    fmt,
    hash::{Hash, Hasher},
    num::NonZero,
    ops::Deref,
};

use aldebaran_logic::prelude::{AsAssert, Assert, Choose, Input};

use aldebaran_source::{
    line::{LineBreak, SourceLines},
    prelude::{Component, SourceDissect, SourceIter},
    terminate::{Outcome, Terminator, TerminatorContext},
};

use aldebaran_baseline::span::Span;

use self::{branch::Branch, error::Expected};

/// A cursor over a [`Source`] `S`.
///
/// This cursor keeps track of its position on the source and provides methods for reading, and matching against its individual components.
///
/// [`Source`]: aldebaran_source::source::Source
// NOTE(invariant): None.
pub struct Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// The position of the cursor on the source.
    source_index: usize,

    /// The source code being traversed.
    source_code: &'source S,
}

impl<'source, S> Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Create a new [`Cursor`] at the beginning of the borrowed source.
    #[inline]
    pub const fn create(source_code: &'source S) -> Self {
        Self::at(source_code, usize::MIN)
    }

    /// Create a [`Cursor`] positioned at one source boundary.
    #[inline]
    #[must_use]
    pub const fn at(source_code: &'source S, source_index: usize) -> Self {
        Self { source_index, source_code }
    }
}

impl<'source, S> Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Retrieve the index of this [`Cursor`] respective to the source code.
    #[inline]
    pub const fn index(&self) -> usize {
        let &Self { source_index, .. } = self;

        source_index
    }

    /// Retrieve the source code of this [`Cursor`].
    #[inline]
    pub const fn source(&self) -> &'source S {
        let &Self { source_code, .. } = self;

        source_code
    }
}

impl<'source, S> Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Dissect this [`Cursor`] as per the given [`Span`].
    ///
    /// This abides by the same rules as [`SourceDissect::dissect`].
    #[inline]
    pub fn slice(&self, span: Span) -> Cursor<'source, S>
    where
        S: SourceDissect<'source, View<'source> = &'source S>,
    {
        let &Self { source_code, .. } = self;

        let source_code: &'source S = source_code.dissect(span);
        let source_index = span.start();

        Cursor { source_index, source_code }
    }

    /// Mutably borrow this [`Cursor`], creating a new [`Branch`] from it.
    ///
    /// Branching allows for arbitrary speculative operations to be performed on
    /// a [`Cursor`], and commiting the changes if necessary.
    #[inline]
    pub fn branch<'borrow>(&'borrow mut self) -> Branch<'source, 'borrow, S>
    where
        'source: 'borrow,
    {
        Branch::from(self)
    }
}

impl<'source, S> Cursor<'source, S>
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
        let _: Option<S::Component> = self.any();
    }
}

impl<'source, S> Cursor<'source, S>
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
        self.expect_is_next_spanned(target_assert).map(|(component, ..)| component)
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
        let &Self { source_index, .. } = self;

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
        let &Self { source_index, .. } = self;

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

impl<'source, S> Cursor<'source, S>
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

impl<'source, S> Cursor<'source, S>
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
        while self.optional(&target_assert).is_some() {}
    }

    /// Counts the number of components that satisfy the target [`Assert`].
    ///
    /// This consumes said components, irrevocably.
    #[inline]
    pub fn count<A, T>(&mut self, target_assert: T) -> usize
    where
        A: Assert<S::Component>,
        for<'b> &'b T: AsAssert<S::Component, A>,
        S::Component: Input,
    {
        let mut target_count = usize::MIN;

        while self.is_next(&target_assert) {
            let _: Option<S::Component> = self.any();

            target_count += 1;
        }

        target_count
    }
}

impl<'source, S> Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + SourceLines<'source> + ?Sized,
{
    /// Consume the current line in the [`Cursor`] using the default source-defined [`Terminator`].
    ///
    /// This returns a 2-tuple indicating the same-line content and the [`LineBreak`], if any.
    #[inline]
    pub fn line(&mut self) -> (Option<Span>, Option<LineBreak<'source, S>>) {
        let mut target_termine = S::DEFAULT_TERMINATOR;

        Self::line_with(self, &mut target_termine)
    }

    /// Consume the current line in the [`Cursor`] using the supplied [`Terminator`].
    ///
    /// This returns a 2-tuple indicating the same-line content and the [`LineBreak`], if any.
    #[inline]
    pub fn line_with<T>(&mut self, target_terminate: &mut T) -> (Option<Span>, Option<LineBreak<'source, S>>)
    where
        T: Terminator<'source, S>,
    {
        let &mut Self {
            source_index: content_start,
            source_code,
        } = self;

        'a: {
            while let Some((target_component, target_span)) = self.any_spanned() {
                let target_context = TerminatorContext::raw(source_code, target_component, target_span);

                if let Outcome::Break(target_break) = target_terminate.state(&target_context) {
                    let content_end = target_break.span().start();

                    let content_bound = content_end
                        .checked_sub(content_start)
                        .and_then(NonZero::new)
                        .map(|content_size| Span::new(content_start, content_size));

                    break 'a (content_bound, Some(target_break));
                }
            }

            let &mut Self {
                source_index: content_end, ..
            } = self;

            (
                NonZero::new(content_end - content_start).map(|content_size| Span::new(content_start, content_size)),
                None,
            )
        }
    }
}

impl<'source, S> Deref for Cursor<'source, S>
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

impl<'source, S> Default for Cursor<'source, S>
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

impl<'source, S: Hash> Hash for Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let &Self { source_index, source_code } = self;

        source_index.hash(state);
        source_code.hash(state);
    }
}

impl<'source, S> Eq for Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    S: Eq,
{
}

impl<'source, S> PartialEq for Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    S: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { source_index, source_code } = self;

        let Self {
            source_index: other_index,
            source_code: other_code,
        } = other;

        source_index == other_index && source_code == other_code
    }
}

impl<'source, S> Copy for Cursor<'source, S> where S: SourceDissect<'source> + SourceIter<'source> + ?Sized {}

impl<'source, S> Clone for Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'source, S> fmt::Debug for Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
    S::Component: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cursor @ {} -> {:?}", self.index(), self.peek())
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`cursor`] module.
    //!
    //! [`cursor`]: super

    use core::num::NonZero;

    use aldebaran_report::annotated::Annotated;

    use aldebaran_source::terminate::{Outcome, Terminator, TerminatorContext, Two};

    use aldebaran_baseline::span::Span;

    use crate::prelude::AsciiAlphanumeric;

    use super::Cursor;

    #[test]
    fn validation_reports_a_mismatch_without_consuming_it() {
        let mut text = Cursor::create("α7");
        let error = text.validate_is_next(AsciiAlphanumeric).expect_err("Greek letters are not ASCII");

        assert_eq!(Annotated::target(&error), Span::unit(0));
        assert_eq!(text.index(), 0);
        assert_eq!(text.any(), Some('α'));
        assert_eq!(text.validate_is_next(AsciiAlphanumeric), Ok('7'));
        assert_eq!(text.index(), 2);
        assert_eq!(text.peek(), Some('7'));
    }

    #[test]
    fn consuming_mismatch_preserves_the_rejected_component_extent() {
        let mut text = Cursor::create("α7");
        let error = text.expect_is_next(AsciiAlphanumeric).expect_err("Greek letters are not ASCII");
        let width = NonZero::new(2).expect("the UTF-8 character occupies two bytes");

        assert_eq!(Annotated::target(&error), Span::new(0, width));
        assert_eq!(text.index(), 2);
        assert_eq!(text.expect_is_next_spanned(AsciiAlphanumeric), Ok(('7', Span::unit(2))));
    }

    #[test]
    fn optional_matching_does_not_consume_a_mismatch_or_advance_past_eof() {
        let mut text = Cursor::create("!7");

        assert_eq!(text.optional_spanned(AsciiAlphanumeric), None);
        assert_eq!(text.index(), 0);
        assert_eq!(text.any(), Some('!'));
        assert_eq!(text.optional_spanned(AsciiAlphanumeric), Some(('7', Span::unit(1))));

        for _ in 0..3 {
            assert_eq!(text.optional(AsciiAlphanumeric), None);
            assert_eq!(text.optional_spanned(AsciiAlphanumeric), None);
            assert_eq!(text.index(), 2);
        }
    }

    #[test]
    fn ignored_span_covers_utf8_bytes_and_leaves_the_first_mismatch() {
        let mut text = Cursor::create("αα!");
        let width = NonZero::new(4).expect("two Greek letters occupy four bytes");

        assert_eq!(text.ignore_while_spanned('α'), Some(Span::new(0, width)));
        assert_eq!(text.index(), 4);
        assert_eq!(text.peek(), Some('!'));
        assert_eq!(text.ignore_while_spanned('α'), None);
        assert_eq!(text.index(), 4);
    }

    #[test]
    fn count_measures_components_rather_than_source_bytes() {
        let mut text = Cursor::create("αα!");

        assert_eq!(text.count('α'), 2);
        assert_eq!(text.index(), 4);
        assert_eq!(text.count('α'), 0);
        assert_eq!(text.peek(), Some('!'));
    }

    #[test]
    fn missing_input_targets_terminal_source_boundary() {
        let source = "a";
        let mut text = Cursor::create(source);

        assert_eq!(text.any(), Some('a'));

        let error = text.expect_is_next(AsciiAlphanumeric).expect_err("source should be exhausted");
        let target = Annotated::target(&error);

        assert_eq!(target.start(), source.len());
        assert_eq!(target.length(), NonZero::<usize>::MIN);
        assert_eq!(target.end(), source.len() + 1);
    }

    #[test]
    fn lookahead_counts_components_and_preserves_width() {
        let mut text = Cursor::create("αb");

        assert_eq!(text.peek_at::<0>(), Some('α'));
        assert_eq!(text.peek_at::<1>(), Some('b'));
        assert_eq!(text.any_spanned(), Some(('α', Span::new(0, NonZero::new(2).expect("width")))));
        assert_eq!(text.index(), 2);
        assert_eq!(text.peek(), Some('b'));
    }

    #[test]
    fn speculative_branch_changes_are_discarded_without_backport() {
        let mut cursor = Cursor::create("abc");

        {
            let mut branch = cursor.branch();

            assert_eq!(branch.any(), Some('a'));
            assert_eq!(branch.any(), Some('b'));
            assert_eq!(branch.index(), 2);
        }

        assert_eq!(cursor.index(), 0, "dropping a branch leaves the parent boundary unchanged");
        assert_eq!(cursor.peek(), Some('a'));
    }

    #[test]
    fn backport_commits_the_speculative_source_boundary() {
        let mut cursor = Cursor::create("abc");
        let mut branch = cursor.branch();

        assert_eq!(branch.any(), Some('a'));
        assert_eq!(branch.any(), Some('b'));
        branch.backport();

        assert_eq!(cursor.index(), 2);
        assert_eq!(cursor.peek(), Some('c'));
    }

    fn span(start: usize, length: usize) -> Span {
        Span::new(start, NonZero::new(length).expect("nonempty span"))
    }

    #[test]
    fn line_consumes_lf_and_returns_remaining_content_at_eof() {
        let mut cursor = Cursor::create("abc\nxyz");

        let (content, line_break) = cursor.line();

        assert_eq!(content, Some(span(0, 3)));
        assert_eq!(line_break.as_ref().map(|b| *b.line_break()), Some("\n"));
        assert_eq!(line_break.as_ref().map(|b| b.span()), Some(Span::unit(3)));
        assert_eq!(cursor.index(), 4);

        let (content, line_break) = cursor.line();

        assert_eq!(content, Some(span(4, 3)));
        assert!(line_break.is_none());
        assert_eq!(cursor.index(), 7);

        assert_eq!(cursor.line(), (None, None));
        assert_eq!(cursor.index(), 7);
    }

    #[test]
    fn line_distinguishes_empty_content_from_exhaustion() {
        let mut cursor = Cursor::create("\n\n");

        for expected_offset in [1, 2] {
            let (content, line_break) = cursor.line();

            assert!(content.is_none());
            assert_eq!(line_break.as_ref().map(|b| *b.line_break()), Some("\n"),);
            assert_eq!(cursor.index(), expected_offset);
        }

        assert_eq!(cursor.line(), (None, None));

        let mut empty = Cursor::create("");
        assert_eq!(empty.line(), (None, None));
        assert_eq!(empty.index(), 0);
    }

    #[test]
    fn line_recognizes_crlf_but_keeps_bare_cr_as_content() {
        let mut cursor = Cursor::create("a\rb\r\nc");

        let (content, line_break) = cursor.line();

        assert_eq!(content, Some(span(0, 3)));
        assert_eq!(line_break.as_ref().map(|b| *b.line_break()), Some("\r\n"),);
        assert_eq!(line_break.as_ref().map(|b| b.span()), Some(span(3, 2)),);
        assert_eq!(cursor.index(), 5);

        let (content, line_break) = cursor.line();

        assert_eq!(content, Some(Span::unit(5)));
        assert!(line_break.is_none());
        assert_eq!(cursor.index(), 6);
    }

    #[test]
    fn line_preserves_utf8_byte_offsets() {
        let mut cursor = Cursor::create("αβ\r\nγ");

        let (content, line_break) = cursor.line();

        assert_eq!(content, Some(span(0, 4)));
        assert_eq!(line_break.as_ref().map(|b| b.span()), Some(span(4, 2)),);
        assert_eq!(cursor.index(), 6);

        let (content, line_break) = cursor.line();

        assert_eq!(content, Some(span(6, 2)));
        assert!(line_break.is_none());
        assert_eq!(cursor.index(), 8);
    }

    #[test]
    fn line_with_reuses_a_custom_terminator() {
        let mut cursor = Cursor::create("a::b::c");
        let mut terminator = Two::tuple((':', ':'));

        let (content, line_break) = cursor.line_with(&mut terminator);

        assert_eq!(content, Some(Span::unit(0)));
        assert_eq!(line_break.as_ref().map(|b| *b.line_break()), Some("::"),);
        assert_eq!(cursor.index(), 3);

        let (content, line_break) = cursor.line_with(&mut terminator);

        assert_eq!(content, Some(Span::unit(3)));
        assert_eq!(line_break.as_ref().map(|b| b.span()), Some(span(4, 2)),);
        assert_eq!(cursor.index(), 6);

        let (content, line_break) = cursor.line_with(&mut terminator);

        assert_eq!(content, Some(Span::unit(6)));
        assert!(line_break.is_none());
        assert_eq!(cursor.index(), 7);

        assert_eq!(cursor.line_with(&mut terminator), (None, None));
    }

    #[test]
    fn line_supports_byte_sources() {
        let mut cursor = Cursor::create(b"a\r\nb\n".as_slice());

        let (content, line_break) = cursor.line();

        assert_eq!(content, Some(Span::unit(0)));
        assert_eq!(line_break.as_ref().map(|b| *b.line_break()), Some(b"\r\n".as_slice()),);
        assert_eq!(cursor.index(), 3);

        let (content, line_break) = cursor.line();

        assert_eq!(content, Some(Span::unit(3)));
        assert_eq!(line_break.as_ref().map(|b| *b.line_break()), Some(b"\n".as_slice()),);
        assert_eq!(cursor.index(), 5);

        assert_eq!(cursor.line(), (None, None));
    }

    #[test]
    fn line_with_handles_a_terminator_started_before_the_cursor() {
        let source = "a\r\n";
        let mut terminator = Two::<char>::CR_LF;

        // The terminator has already observed the CR.
        let context = TerminatorContext::raw(source, '\r', Span::unit(1));

        assert!(matches!(terminator.state(&context), Outcome::Continue(()),));

        let mut cursor = Cursor::at(source, 2);

        let (content, line_break) = cursor.line_with(&mut terminator);

        assert_eq!(content, None);
        assert_eq!(line_break.as_ref().map(|b| b.span()), Some(span(1, 2)),);
        assert_eq!(cursor.index(), 3);
    }
}
