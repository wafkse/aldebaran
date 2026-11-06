//! Unit tests for report metadata validation.

use super::Report;

fn parse(input: &str) -> syn::Result<Report> {
    Report::new(syn::parse_str(input)?)
}

#[test]
fn rejects_malformed_and_duplicate_darling_metadata() {
    let malformed = parse(
        r#"
            #[report(title = 42, message = "bad input")]
            struct Issue(Span);
        "#,
    );
    assert!(malformed.is_err(), "title metadata must be a string literal");

    let duplicate = parse(
        r#"
            #[report(title = "first", title = "second", message = "bad input")]
            struct Issue(Span);
        "#,
    );
    assert!(duplicate.is_err(), "Darling must reject duplicate declarations");
}

#[test]
fn rejects_conflicting_policies_and_invalid_targets() {
    let conflicting = parse(
        r#"
            #[report(title = "static", title_with = project_title, message = "bad input")]
            struct Issue(Span);
        "#,
    );
    assert!(conflicting.is_err(), "static and projected titles are exclusive");

    let invalid_target = parse(
        r#"
            #[report(title = "stored", annotations = missing)]
            struct Issue { annotations: Annotations }
        "#,
    );
    assert!(invalid_target.is_err(), "selected annotation members must exist");

    let invalid_variant = parse(
        r#"
            enum Issue {
                #[report(title = "invalid", message = "bad input")]
                Pair(Span, Span),
            }
        "#,
    );
    assert!(invalid_variant.is_err(), "report alternatives wrap one tuple field");
}

#[test]
fn rejects_conditional_forward_alternatives() {
    for input in [
        "
            #[report(forward)]
            enum Issue {
                #[cfg(unix)]
                Wrapped(InnerIssue),
            }
        ",
        "
            #[report(forward)]
            enum Issue {
                #[cfg_attr(unix, allow(dead_code))]
                Wrapped(InnerIssue),
            }
        ",
        "
            #[report(forward)]
            enum Issue {
                Wrapped(#[cfg(unix)] InnerIssue),
            }
        ",
        "
            #[report(forward)]
            enum Issue {
                Wrapped(#[cfg_attr(unix, allow(dead_code))] InnerIssue),
            }
        ",
    ] {
        let error = parse(input).expect_err("forward alternatives and payloads cannot be conditional");

        assert!(error.to_string().contains("cannot use cfg or cfg_attr"));
    }

    let conflicting = parse(
        r#"
            #[report(forward, title = "ignored")]
            enum Issue {
                Wrapped(InnerIssue),
            }
        "#,
    );

    assert!(conflicting.is_err(), "forward cannot be combined with other report metadata");
}
