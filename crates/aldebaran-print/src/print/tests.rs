extern crate alloc;

use alloc::{format, string::String};
use core::fmt;

use crate::combine::Combine as _;

use super::Print as _;

/// Caller-owned names used to resolve printed indices.
// NOTE(invariant): None.
struct Names(&'static [&'static str]);

/// An index into the external name table.
// NOTE(invariant): None.
struct Name(usize);

impl super::Print for Name {
    type Context = Names;

    fn print_with_context<W>(&self, writer: &mut W, context: &Self::Context) -> fmt::Result
    where
        W: fmt::Write,
    {
        let Self(target_index) = self;

        let Names(target_names) = context;

        writer.write_str(target_names[*target_index])
    }
}

#[test]
fn composition_writes_directly_in_order() {
    let mut output = String::new();

    "ab".sequence('-')
        .sequence('x'.times(3))
        .print(&mut output)
        .expect("writing to a String succeeds");

    assert_eq!(output, "ab-xxx");
}

#[test]
fn contextual_values_do_not_require_default_contexts() {
    let mut output = String::new();
    let context = Names(&["left", "right"]);

    Name(0)
        .aggregate(Name(1))
        .print_with_context(&mut output, &context)
        .expect("writing to a String succeeds");

    assert_eq!(output, "leftright");
}

#[test]
fn default_context_values_can_use_standard_display_formatting() {
    let output = format!("value={}", 42_u8.display());

    assert_eq!(output, "value=42");
}

#[test]
fn contextual_values_can_use_standard_display_formatting() {
    let context = Names(&["left", "right"]);
    let target_name = Name(1);

    let output = format!("name={}", target_name.display_with_context(&context));

    assert_eq!(output, "name=right");
}

#[test]
fn slices_keep_delimiter_order() {
    let mut output = String::new();

    [1_u8, 2, 3].print(&mut output).expect("writing to a String succeeds");

    assert_eq!(output, "[1, 2, 3]");
}
