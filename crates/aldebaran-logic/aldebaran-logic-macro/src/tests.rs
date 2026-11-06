//! Unit tests for assertion attribute parsing and derive helpers.

use syn::{Attribute, parse_quote};

use super::{Param, fresh, pattern::Pattern};

#[test]
fn fresh_parameters_avoid_const_and_raw_identifiers() {
    let generics = parse_quote!(<r#__AssertTy, __AssertTy_, const __AssertTy__: usize>);

    assert_eq!(fresh(&generics, "__AssertTy"), "__AssertTy___");
}

#[test]
fn rejects_malformed_duplicate_and_conflicting_assertion_metadata() {
    let malformed: Attribute = parse_quote!(#[assert(unknown)]);
    assert!(Param::attribute(malformed).is_err());

    let duplicate_key: Attribute = parse_quote!(#[assert(conjunctive, conjunctive)]);
    assert!(Param::attribute(duplicate_key).is_err());

    let conflicting: Attribute = parse_quote!(#[assert(conjunctive, disjunctive)]);
    assert!(Param::attribute(conflicting).is_err());

    let mut duplicate_attributes = vec![parse_quote!(#[assert(conjunctive)]), parse_quote!(#[assert(disjunctive)])];
    assert!(Param::take_attribute(&mut duplicate_attributes).is_err());
}

#[test]
fn rejects_pattern_expectations_with_payload_variants() {
    let input = parse_quote! {
        #[assert(input(Token))]
        enum Expected {
            #[assert(matches(Token::End) as "end")]
            End(u8),
        }
    };

    assert!(Pattern::new(input).is_err());
}

#[test]
fn pattern_descriptions_require_as() {
    let input = parse_quote! {
        #[assert(input(Token))]
        enum Expected {
            #[assert(matches(Token::End), "end")]
            End,
        }
    };

    let error = Pattern::new(input).expect_err("comma syntax is no longer accepted");

    assert!(error.to_string().contains("as"));
}
