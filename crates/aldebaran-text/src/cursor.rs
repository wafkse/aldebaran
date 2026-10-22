//! Source traversal for lexical recognition and recovery. [`Cursor`] preserves
//! source coordinates for diagnostics, [`branch`] makes cursor progress speculative,
//! and [`consume`] separates recognition from construction of lexical values.

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

use aldebaran_baseline::span::Span;

use self::{branch::Branch, error::Expected};

/// Traverses source components while preserving their diagnostic coordinates.
/// Offsets and spans use source units, which are bytes for `str` and byte slices,
/// while lookahead counts components, which are Unicode scalars for `str`.
// NOTE(invariant): None.
pub struct Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Position of the next component in the source's coordinate units.
    source_index: usize,

    /// Source whose coordinate system gives `source_index` its meaning.
    source_code: &'source S,
}

impl<'source, S> Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Begin traversal at offset zero, including for an empty source.
    #[inline]
    pub const fn create(source_code: &'source S) -> Self {
        Self::at(source_code, 0)
    }

    /// Start at an absolute source offset.
    ///
    /// The offset is not validated. Out-of-range offsets may behave as exhaustion,
    /// and offsets inside a `str` scalar can panic during a read.
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
    /// Position of the next component in source units, including the end boundary
    /// after ordinary traversal exhausts the source.
    #[inline]
    pub const fn index(&self) -> usize {
        let &Self { source_index, .. } = self;

        source_index
    }

    /// Source against which cursor offsets and diagnostic spans are interpreted,
    /// available for the original source lifetime.
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
    /// Restrict the source through [`SourceDissect::dissect`] while retaining
    /// `span.start()` as the new cursor's index.
    ///
    /// Built-in string and byte views have coordinates beginning at zero, so a
    /// nonzero start also skips that many units within the selected view and may
    /// exhaust it. String views require UTF-8 boundaries for the selected endpoints
    /// and for subsequent reads within the view.
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

    /// Start speculative traversal at the current offset.
    ///
    /// Dropping the branch discards its movement. [`Branch::backport`] commits it.
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
    /// Whether traversal has no next component at the current source offset.
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.peek().is_none()
    }

    /// Inspect the current component without changing its diagnostic position.
    #[inline]
    pub fn peek(&self) -> Option<S::Component> {
        self.peek_at::<0>()
    }

    /// Inspect the component `N` positions after the next component.
    ///
    /// `N` counts components. `N = 0` is equivalent to [`Self::peek`].
    #[inline]
    pub fn peek_at<const N: usize>(&self) -> Option<S::Component> {
        let &Self { source_index, source_code } = self;

        source_code.offset(source_index)?.iter().nth(N)
    }

    /// Advance past the next component by its source width, leaving the offset
    /// unchanged when input is exhausted.
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

    /// Associate the consumed component with its original source extent.
    /// Exhaustion leaves the offset unchanged.
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

    /// Advance past a component without retaining it, leaving exhaustion unchanged.
    #[inline]
    pub fn skip(&mut self) {
        let _: Option<S::Component> = self.any();
    }
}

impl<'source, S> Cursor<'source, S>
where
    S: SourceDissect<'source> + SourceIter<'source> + ?Sized,
{
    /// Consume the next component and check it against an [`Assert`].
    ///
    /// A mismatching component remains consumed, while exhaustion is reported at
    /// a unit span at the current offset.
    #[inline]
    pub fn expect_is_next<A, T>(&mut self, target_assert: T) -> Result<S::Component, Expected<S::Component, A>>
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component> + Clone,
        S::Component: Input,
    {
        self.expect_is_next_spanned(target_assert).map(|(component, _)| component)
    }

    /// Consume the next component and classify it with [`Choose`].
    ///
    /// A mismatch remains consumed, while exhaustion is reported at a unit span
    /// at the current offset.
    #[inline]
    pub fn expected_is_next<A>(&mut self, target_assert: A) -> Result<A::Unit, Expected<S::Component, A>>
    where
        A: Choose<S::Component> + Clone,
        S::Component: Input,
    {
        self.expected_is_next_spanned(target_assert).map(|(unit, _)| unit)
    }

    /// Check the next component without consuming it.
    ///
    /// A mismatch or exhaustion is reported at a unit span at the current offset.
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

    /// Check the next component, accepting either a match or exhaustion.
    ///
    /// A mismatch is reported at a unit span at the current offset. The cursor is
    /// not advanced.
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

    /// Consume the next component only when it satisfies the assertion.
    ///
    /// A mismatch or exhaustion leaves the cursor unchanged.
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
    /// Require the next component to satisfy the assertion and preserve its source extent.
    ///
    /// A mismatch is consumed and reported at that component's span, while exhaustion
    /// is reported at a unit span at the current offset.
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

    /// Classify the consumed component while preserving its original source extent.
    ///
    /// A mismatch is consumed and reported at that component's span, while exhaustion
    /// is reported at a unit span at the current offset.
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

    /// Consume a matching component and return its source [`Span`].
    ///
    /// A mismatch or exhaustion returns `None` without advancing.
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

    /// Consume a maximal matching run and identify its source extent, leaving the
    /// first mismatch unconsumed and producing no span for an empty run.
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
    /// Acceptance at exhaustion is the caller-selected policy `D`.
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

    /// Test the component `N` positions after the next component.
    ///
    /// `N` counts components. Missing input returns `false`.
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

    /// Whether the next component satisfies the assertion, with exhaustion treated
    /// as rejection and the cursor position preserved.
    #[inline]
    pub fn is_next<A, T>(&self, target_assert: T) -> bool
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component>,
        S::Component: Input,
    {
        self.is_at::<0, A, T>(target_assert)
    }

    /// Return whether the next component matches or input is exhausted.
    #[inline]
    pub fn is_next_or_eof<A, T>(&self, target_assert: T) -> bool
    where
        T: AsAssert<S::Component, A>,
        A: Assert<S::Component>,
        S::Component: Input,
    {
        self.is_next_or_default::<true, A, T>(target_assert)
    }

    /// Discard a maximal matching run, leaving the first mismatch unconsumed.
    #[inline]
    pub fn ignore_while<A, T>(&mut self, target_assert: T)
    where
        A: Assert<S::Component>,
        for<'b> &'b T: AsAssert<S::Component, A>,
        S::Component: Input,
    {
        while self.optional(&target_assert).is_some() {}
    }

    /// Count consumed matches in components rather than source units, leaving the
    /// first mismatch unconsumed and yielding zero for an empty run.
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
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { source_index, source_code } = self;

        f.debug_struct("Cursor")
            .field("source_index", source_index)
            .field("source_code", source_code)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use core::num::NonZero;

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

    use aldebaran_baseline::span::Span;

    use crate::prelude::{AsciiAlphanumeric, Digit};

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

    #[test]
    fn a() {
        let s = "bb";

        let mut t = Cursor::create(s);

        let first_error = t.expect_is_next(AsciiAlphanumeric.or('_').and(Digit)).expect_err("valid");

        let err = t.expect_is_next(AsciiAlphanumeric.or('_').and(Digit)).expect_err("valid");

        assert_eq!(Annotated::target(&first_error), Span::unit(0));
        assert_eq!(Annotated::target(&err), Span::unit(1));
        assert_eq!(t.index(), s.len());

        let mut target_sink = String::new();

        let report = err.attach(s);
        let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut target_sink);

        renderer
            .render_mut_with_input(Present::with_input(FancySettings::level(Fancyness::Outblown)), &report)
            .expect("failed to render error");

        println!("{}", target_sink);
    }
}
