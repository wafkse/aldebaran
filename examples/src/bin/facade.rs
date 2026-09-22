//! Facade-focused lexical scanning and diagnostic access.

use aldebaran::{
    prelude::{Annotated, Report},
    text::prelude::{AsciiDigit, Text},
};

fn main() {
    let mut text = Text::create("9x");
    let digit = text.expect_is_next(AsciiDigit).expect("the first character is a digit");
    let error = text.expect_is_next(AsciiDigit).expect_err("the second character is not a digit");

    assert_eq!(digit, '9');
    assert_eq!(Report::title(&error), "unexpected character");

    println!("parsed digit {digit}");
    println!("next error targets {:?}", Annotated::target(&error));
}
