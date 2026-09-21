//! Human-readable visualization of source-shaped values.
//!
//! [`Visualize`] is intentionally distinct from ordinary printing. A byte, for
//! example, prints as a number but visualizes as source text. Implementations
//! write directly into a formatting sink and never allocate an intermediate
//! string.

use core::fmt;

/// A value with a human-readable, potentially lossy textual visualization.
pub trait Visualize {
    /// Write this value's visualization directly into `writer`.
    ///
    /// # Errors
    ///
    /// Returns the formatting error produced by `writer`.
    fn visualize<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write;

    /// Borrow this value through standard formatting traits.
    #[inline]
    #[must_use]
    fn visual(&self) -> impl fmt::Display + fmt::Debug + '_ {
        Visual(self)
    }
}

/// Borrowed standard-formatting view of one [`Visualize`] value.
///
/// Both [`fmt::Display`] and [`fmt::Debug`] use the same visualization. The
/// wrapper remains private because callers only need those formatting
/// capabilities.
#[repr(transparent)]
struct Visual<'a, V>(&'a V)
where
    V: Visualize + ?Sized;

impl<V> fmt::Display for Visual<'_, V>
where
    V: Visualize + ?Sized,
{
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self(target_value) = self;

        target_value.visualize(formatter)
    }
}

impl<V> fmt::Debug for Visual<'_, V>
where
    V: Visualize + ?Sized,
{
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Self(target_value) = self;

        target_value.visualize(formatter)
    }
}

impl<V> Visualize for &V
where
    V: Visualize + ?Sized,
{
    #[inline]
    fn visualize<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        V::visualize(self, writer)
    }
}

impl Visualize for str {
    #[inline]
    fn visualize<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        writer.write_str(self)
    }
}

impl Visualize for [u8] {
    #[inline]
    fn visualize<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        for byte in self.iter().copied() {
            let target_char = if byte.is_ascii() {
                byte as char
            } else {
                char::REPLACEMENT_CHARACTER
            };

            writer.write_char(target_char)?;
        }

        Ok(())
    }
}

impl Visualize for char {
    #[inline]
    fn visualize<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        writer.write_char(*self)
    }
}

impl Visualize for u8 {
    #[inline]
    fn visualize<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        core::slice::from_ref(self).visualize(writer)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::{format, string::String};

    use super::Visualize as _;

    #[test]
    fn bytes_visualize_as_source_text() {
        let input = [b'A', 0xff, b'Z'];
        let mut output = String::new();

        input.as_slice().visualize(&mut output).unwrap();

        assert_eq!(output, "A�Z");
    }

    #[test]
    fn visual_view_shares_display_and_debug_output() {
        let input = [b'A', 0xff];
        let visual = input.as_slice().visual();

        assert_eq!(format!("{visual}"), "A�");
        assert_eq!(format!("{visual:?}"), "A�");
    }
}
