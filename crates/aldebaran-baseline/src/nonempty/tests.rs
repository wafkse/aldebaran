use alloc::{string::String, vec, vec::Vec};
use core::{num::NonZero, ptr};

use super::{Nonempty, reference::RefNonempty};

#[test]
fn vector_conversion_preserves_empty_singleton_and_ordered_inputs() {
    assert_eq!(Nonempty::<u32>::from_vec(Vec::new()), None);
    assert_eq!(Nonempty::from_vec(vec![7]), Some(Nonempty::single(7)));

    let values = Nonempty::from_vec(vec![1, 2, 3, 4]).expect("nonempty vector");

    assert_eq!(*values.first(), 1);
    assert_eq!(values.rest(), [2, 3, 4]);
    assert_eq!(values.iter().copied().collect::<Vec<_>>(), [1, 2, 3, 4]);
}

#[test]
fn tail_growth_and_removal_retain_the_required_value() {
    let mut values = Nonempty::single(1);

    values.list_mut().extend([2, 3]);
    *values.first_mut() = 4;
    values.rest_mut().reverse();

    for value in values.iter_mut() {
        *value += 10;
    }

    assert_eq!(values.iter().copied().collect::<Vec<_>>(), [14, 13, 12]);

    values.list_mut().clear();

    assert_eq!(values.len(), NonZero::<usize>::MIN);
    assert_eq!(values.iter().copied().collect::<Vec<_>>(), [14]);
}

#[test]
fn borrowed_views_share_owned_elements_and_preserve_order() {
    let first = String::from("primary");
    let rest = vec![String::from("related"), String::from("last")];
    let values = Nonempty::multiple(first, rest);
    let view = values.as_ref();

    assert!(ptr::eq(view.first(), values.first()), "the view must share the required value");
    assert!(ptr::eq(view.rest(), values.rest()), "the view must share the tail");
    assert_eq!(view.len(), NonZero::new(3).expect("three elements"));
    assert_eq!(view.iter().map(String::as_str).collect::<Vec<_>>(), ["primary", "related", "last"]);
}

#[test]
fn iterators_survive_borrowed_views_and_remain_exhausted() {
    let first = 1;
    let rest = [2, 3];
    let mut values = {
        let view = RefNonempty::multiple(&first, &rest);

        view.iter()
    };

    assert_eq!(values.next_back(), Some(&3));
    assert_eq!(values.next(), Some(&1));
    assert_eq!(values.next_back(), Some(&2));
    assert_eq!(values.next(), None);
    assert_eq!(values.next_back(), None);
    assert_eq!(values.next(), None);
}

#[test]
fn copying_a_borrowed_view_does_not_require_copy_elements() {
    /// A value that deliberately implements neither `Copy` nor `Clone`.
    #[derive(Debug, PartialEq, Eq)]
    // NOTE(invariant): None.
    struct Value(u32);

    let first = Value(1);
    let rest = [Value(2)];
    let view = RefNonempty::tuple((&first, &rest));
    let copy = view;

    assert!(
        ptr::eq(view.first(), copy.first()),
        "copying the view must retain its input references"
    );
    assert_eq!(copy.into_raw_parts(), (&first, rest.as_slice()));
    assert_eq!(RefNonempty::single(&first).iter().collect::<Vec<_>>(), [&first]);
}

#[test]
fn zero_sized_tail_lengths_saturate_without_losing_nonemptiness() {
    let first = ();
    let rest = [(); usize::MAX];
    let view = RefNonempty::multiple(&first, &rest);

    assert_eq!(view.len().get(), usize::MAX);
}
