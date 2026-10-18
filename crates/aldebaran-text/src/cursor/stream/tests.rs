use aldebaran_grammar::stream::TokenStream;

use crate::cursor::{Cursor, consume::Lex, internment::Internment};

use super::LexStream;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// NOTE(invariant): None.
struct Counted(u8);

impl<'source> Lex<'source, [u8]> for Counted {
    type Input<'input>
        = u8
    where
        'source: 'input;

    type Context = usize;

    type Error = ();

    #[inline]
    fn pass<'input>(text: &'input mut Cursor<'source, [u8]>) -> Result<Self::Input<'input>, Self::Error>
    where
        'source: 'input,
    {
        text.any().ok_or(())
    }

    #[inline]
    fn build_with_context<'input>(byte: Self::Input<'input>, count: &mut Self::Context) -> Result<Self, Self::Error>
    where
        'source: 'input,
    {
        *count = count.saturating_add(1);

        Ok(Self(byte))
    }
}

#[test]
fn stream_reuses_one_context_and_stops_at_source_exhaustion() {
    let source = b"ab".as_slice();
    let mut count = 0_usize;

    {
        let internment = Internment::new(source, &mut count);
        let mut stream = LexStream::<[u8], Counted>::new(internment);

        assert_eq!(TokenStream::next(&mut stream), Ok(Some(Counted(b'a'))));
        assert_eq!(TokenStream::next(&mut stream), Ok(Some(Counted(b'b'))));
        assert_eq!(TokenStream::next(&mut stream), Ok(None));
    }

    assert_eq!(count, 2);
}

#[test]
fn empty_source_never_invokes_the_lexer() {
    let mut count = 0_usize;

    {
        let source = b"".as_slice();
        let internment = Internment::new(source, &mut count);
        let mut stream = LexStream::<[u8], Counted>::new(internment);

        for _ in 0..3 {
            assert_eq!(stream.next(), Ok(None));
        }
    }

    assert_eq!(count, 0);
}
