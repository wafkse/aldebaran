//! Writes repeated frame, spacing, and marker glyphs through a bounded stack buffer.

use core::{fmt, str};

use aldebaran_print::prelude::Print;

/// Writes one glyph repeatedly in bounded chunks without allocating an output buffer.
// NOTE(invariant): None.
pub(in crate::render::textual::offload::fancy) struct GlyphRun {
    /// Glyph selected by the existing presentation settings.
    pub(in crate::render::textual::offload::fancy) glyph: char,

    /// Number of copies to emit.
    pub(in crate::render::textual::offload::fancy) count: u32,
}

impl Print for GlyphRun {
    type Context = ();

    fn print_with_context<W>(&self, writer: &mut W, _: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let &Self { glyph, mut count } = self;

        if let u32::MIN = count {
            Ok(())
        } else {
            let mut encoded = [u8::MIN; 4];
            let glyph = glyph.encode_utf8(&mut encoded);
            let mut buffer = [u8::MIN; 256];
            let copy_count = buffer.len() / glyph.len();
            let length = copy_count * glyph.len();

            for slot in buffer.chunks_exact_mut(glyph.len()) {
                slot.copy_from_slice(glyph.as_bytes());
            }

            let text = str::from_utf8(&buffer[..length]).map_err(|_error| fmt::Error)?;

            while count >= copy_count as u32 {
                writer.write_str(text)?;
                count -= copy_count as u32;
            }

            match count {
                u32::MIN => Ok(()),
                _ => writer.write_str(&text[..count as usize * glyph.len()]),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests module for the [`run`] module.
    //!
    //! [`run`]: super

    use aldebaran_print::prelude::Print;

    use super::GlyphRun;

    #[test]
    fn utf8_runs_remain_whole_across_chunk_boundaries() {
        for glyph in [' ', '^', 'é', '─', '🦀'] {
            for count in [u32::MIN, 1, 63, 64, 85, 86, 255, 256, 257, 1000] {
                let run = GlyphRun { glyph, count };
                let mut output = String::new();

                run.print(&mut output).expect("print UTF-8 run");

                assert_eq!(
                    output,
                    glyph.to_string().repeat(count as usize),
                    "chunk boundaries must preserve every glyph"
                );
            }
        }
    }
}
