//! Byte-wise rendering for source-oriented output.
//!
//! For `[u8]`, ASCII bytes are copied as characters and each non-ASCII byte
//! becomes U+FFFD. The view preserves byte boundaries instead of decoding
//! UTF-8, and control characters remain literal.

use core::{fmt, slice};

/// Source content suitable for direct visualization without contextual name lookup.
/// Built-in byte views preserve byte boundaries, replacing each non-ASCII byte independently.
pub trait Visualize {
    /// Render the source view without requiring an intermediate string.
    ///
    /// # Errors
    ///
    /// Returns the formatting error produced by `writer`. Earlier writes are
    /// not rolled back if a later write fails.
    fn visualize<W>(&self, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write;

    /// Return an adapter that uses [`Self::visualize`] for [`fmt::Display`] and
    /// [`fmt::Debug`].
    #[inline]
    #[must_use]
    fn visual(&self) -> impl fmt::Display + fmt::Debug {
        Visual(self)
    }
}

/// Standard formatting that shares the visualization for display and debug.
/// Debug output adds no quoting or escaping of its own.
#[repr(transparent)]
// NOTE(invariant): None.
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
        for target_byte in self.iter().copied() {
            let target_char = if target_byte.is_ascii() {
                target_byte as char
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
        slice::from_ref(self).visualize(writer)
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

        assert!(input.as_slice().visualize(&mut output).is_ok());

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
