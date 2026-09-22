use core::fmt::Write as _;

use aldebaran_ansi::prelude::{Color, Paintable, Style};
use aldebaran_print::prelude::{Combine, Print};
use aldebaran_style::prelude::Stylus;
use aldebaran_visualize::prelude::Visualize;

fn main() {
    let message = "Aldebaran".aggregate(" compiler framework");
    let mut plain = String::new();
    message.print(&mut plain).expect("string formatting is infallible");
    assert_eq!(plain, "Aldebaran compiler framework");

    let bytes = [b'A', b'B', 0xff, b'C'];
    let visualized = format!("{}", bytes.as_slice().visual());
    assert_eq!(visualized, "AB�C");

    let mut ansi = String::new();
    "warning"
        .bright_yellow()
        .bold()
        .print(&mut ansi)
        .expect("string formatting is infallible");

    let mut style = Style::empty();
    let _ = style.foreground_replace(Color::CYAN);

    let mut direct = String::new();
    style
        .style(&mut direct, |writer| writer.write_str("styled directly"))
        .expect("string formatting is infallible")
        .expect("the styled write succeeds");

    println!("{plain}");
    println!("visualized bytes {visualized}");
    println!("ANSI encoded output {ansi:?}");
    println!("stylus output {direct}");
}
