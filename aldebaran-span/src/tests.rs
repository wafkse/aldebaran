use core::num::NonZeroUsize;

use crate::span::Span;

#[test]
fn test_span_new() {
    let length = NonZeroUsize::new(5).unwrap();
    let span = Span::new(10, length);
    assert_eq!(span.start(), 10);
    assert_eq!(span.length(), length);
}

#[test]
fn test_span_unit() {
    let span = Span::unit(10);
    assert_eq!(span.start(), 10);
    assert_eq!(span.length().get(), 1);
}

#[test]
fn test_span_start() {
    let length = NonZeroUsize::new(5).unwrap();
    let span = Span::new(10, length);
    assert_eq!(span.start(), 10);
}

#[test]
fn test_span_end() {
    let length = NonZeroUsize::new(5).unwrap();
    let span = Span::new(10, length);
    assert_eq!(span.end(), 15);
}

#[test]
fn test_span_contains() {
    let length = NonZeroUsize::new(5).unwrap();
    let span = Span::new(10, length);
    assert!(span.contains(12));
    assert!(!span.contains(15));
}

#[test]
fn test_span_within() {
    let length1 = NonZeroUsize::new(5).unwrap();
    let span1 = Span::new(10, length1);
    let length2 = NonZeroUsize::new(10).unwrap();
    let span2 = Span::new(5, length2);
    assert!(span1.within(span2));
    assert!(!span2.within(span1));
}

#[test]
fn test_span_overlaps() {
    let length1 = NonZeroUsize::new(5).unwrap();
    let span1 = Span::new(10, length1);
    let length2 = NonZeroUsize::new(10).unwrap();
    let span2 = Span::new(12, length2);
    assert!(span1.overlaps(span2));
    assert!(span2.overlaps(span1));
}

#[test]
fn test_span_superset() {
    let length1 = NonZeroUsize::new(5).unwrap();
    let span1 = Span::new(10, length1);
    let length2 = NonZeroUsize::new(10).unwrap();
    let span2 = Span::new(5, length2);
    let superset = span1.superset(span2);
    assert_eq!(superset.start(), 5);
    assert_eq!(superset.end(), 15);
}

#[test]
fn test_span_intersect() {
    let length1 = NonZeroUsize::new(5).unwrap();
    let span1 = Span::new(10, length1);
    let length2 = NonZeroUsize::new(10).unwrap();
    let span2 = Span::new(12, length2);
    let intersection = span1.intersect(span2).unwrap();

    assert_eq!(intersection.start(), 12);
    assert_eq!(intersection.end(), 15);
}

#[test]
fn test_span_englobes() {
    let length1 = NonZeroUsize::new(10).unwrap();
    let span1 = Span::new(5, length1);
    let length2 = NonZeroUsize::new(5).unwrap();
    let span2 = Span::new(10, length2);
    assert!(span1.englobes(span2));
    assert!(!span2.englobes(span1));
}

#[test]
fn test_span_range() {
    let length = NonZeroUsize::new(5).unwrap();
    let span = Span::new(10, length);
    let range = span.range();
    assert_eq!(range.start, 10);
    assert_eq!(range.end, 15);
}
