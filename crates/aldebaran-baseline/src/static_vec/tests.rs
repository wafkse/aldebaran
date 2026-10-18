use super::StaticVec;

#[test]
fn emptiness_tracks_push_and_pop() {
    let mut values = StaticVec::<u32, 1>::empty();

    assert!(values.is_empty(), "new storage must be empty");
    assert_eq!(values.push(7), Ok(()));
    assert!(!values.is_empty(), "a stored element makes the vector nonempty");
    assert_eq!(values.push(8), Err(8));
    assert_eq!(values.pop(), Some(7));
    assert!(values.is_empty(), "popping the last element empties the vector");
}

#[test]
fn zero_capacity_stays_empty() {
    let mut values = StaticVec::<u32, 0>::empty();

    assert_eq!(values.push(7), Err(7));
    assert_eq!(values.pop(), None);
    assert!(values.is_empty(), "zero-capacity storage cannot contain elements");
}

#[test]
fn only_the_initialized_prefix_is_dropped() {
    use alloc::rc::Rc;
    use core::cell::Cell;

    // NOTE(invariant): None.
    struct DropCounter(Rc<Cell<usize>>);

    impl Drop for DropCounter {
        fn drop(&mut self) {
            let Self(counter) = self;

            counter.set(counter.get() + 1);
        }
    }

    let drops = Rc::new(Cell::new(0));
    let mut values = StaticVec::<DropCounter, 3>::empty();

    assert!(matches!(values.push(DropCounter(Rc::clone(&drops))), Ok(())));
    assert!(matches!(values.push(DropCounter(Rc::clone(&drops))), Ok(())));
    assert!(matches!(values.push(DropCounter(Rc::clone(&drops))), Ok(())));
    let popped = values.pop().expect("the last initialized value is available");

    drop(values);
    assert_eq!(drops.get(), 2, "dropping the vector drops only its live prefix");

    drop(popped);
    assert_eq!(drops.get(), 3, "a popped value is owned by the caller");
}
