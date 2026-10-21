//! Assertion contracts preserved across generic parameters and field shapes.

#[cfg(test)]
mod tests {
    use core::{cell::RefCell, fmt};

    use aldebaran_logic::{
        fmt::{BinaryConnective, Connective, Formatter, Precedence, Segment},
        prelude::{Assert, Choose},
    };

    /// Requires every operand to accept while preserving caller-selected parameter names.
    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
    #[assert(conjunctive)]
    // NOTE(invariant): None.
    struct Colliding<__AssertTy, __AssertTy_, __ChooseTy, __ChooseTy_>(__AssertTy, __AssertTy_, __ChooseTy, __ChooseTy_);

    /// Accepts `x` regardless of payload, preserving caller-selected parameter names.
    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
    #[assert(use 'x' type char)]
    // NOTE(invariant): None.
    struct CollidingExpression<__AssertTy, __ChooseTy>(__AssertTy, __ChooseTy);

    /// Preserves caller-selected parameter names while assigning rules to alternatives.
    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
    enum CollidingEnum<__AssertTy, __ChooseTy> {
        /// Both payload predicates must accept the same input.
        #[assert(conjunctive)]
        Fields(__AssertTy, __ChooseTy),
    }

    #[test]
    fn derives_avoid_existing_generic_names() {
        let value = Colliding('x', 'x', 'x', 'x');
        assert!(value.assert('x'));
        assert_eq!(value.choose('x'), Some(value.clone()));
        assert_eq!(value.choose('y'), None);

        let expression = CollidingExpression(0_u8, 1_u8);
        assert!(expression.assert('x'));
        assert_eq!(expression.choose('x'), Some(expression));

        let variant = CollidingEnum::Fields('x', 'x');
        assert!(variant.assert('x'));
        assert_eq!(variant.choose('x'), Some(variant));
    }

    /// Accepts characters equal to any borrowed candidate without taking ownership.
    #[derive(Debug, Clone, PartialEq, Eq, Choose)]
    // NOTE(invariant): None.
    struct Borrowed<'a, T = char, const N: usize = 1>(&'a [T; N])
    where
        T: Clone + PartialEq<char>;

    impl<T, const N: usize> Assert<char> for Borrowed<'_, T, N>
    where
        T: Clone + PartialEq<char>,
    {
        fn assert(&self, input: char) -> bool {
            let Self(values) = self;

            values.iter().any(|value| *value == input)
        }

        fn output_with<W, F>(_: &Self, writer: &mut W, _: Precedence) -> fmt::Result
        where
            W: fmt::Write,
            F: Formatter,
        {
            writer.write_str("p")
        }
    }

    /// Requires acceptance by a borrowed candidate set and classifies as itself.
    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
    #[assert(conjunctive)]
    // NOTE(invariant): None.
    struct Named<'a, T = char, const N: usize = 1>
    where
        T: Clone,
        T: PartialEq<char>,
    {
        /// Candidate set defining the accepted characters.
        predicate: Borrowed<'a, T, N>,
    }

    /// Accepts a borrowed candidate match and classifies as itself.
    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
    #[assert(disjunctive)]
    // NOTE(invariant): None.
    struct Tuple<'a, T = char, const N: usize = 1>(Borrowed<'a, T, N>)
    where
        T: Clone + PartialEq<char>;

    /// Accepts `x` independently of the borrowed payload and classifies as itself.
    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
    #[assert(use 'x' type char)]
    // NOTE(invariant): None.
    struct Expression<'a, T = char, const N: usize = 1>(&'a [T; N])
    where
        T: Clone + PartialEq<char>;

    /// Alternatives that preserve borrowing while classifying accepted inputs as themselves.
    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
    enum Generic<'a, T = char, const N: usize = 1>
    where
        T: Clone,
        T: PartialEq<char>,
    {
        /// Requires acceptance by the borrowed candidate set.
        #[assert(conjunctive)]
        Named {
            /// Candidate set defining the accepted characters.
            predicate: Borrowed<'a, T, N>,
        },
        /// Accepts a match from the borrowed candidate set.
        #[assert(disjunctive)]
        Tuple(Borrowed<'a, T, N>),
        /// Uses the borrowed candidates as its delegated acceptance rule.
        #[assert(use Borrowed(values) type Borrowed<'a, T, N>)]
        Expression {
            /// Candidate characters available to the delegated predicate.
            values: &'a [T; N],
        },
    }

    /// Requires `x` to classify as the supplied value and `z` to be rejected.
    fn check_choice<A>(value: &A)
    where
        A: Choose<char, Unit = A> + PartialEq + fmt::Debug,
    {
        assert_eq!(value.choose('x').as_ref(), Some(value));
        assert_eq!(value.choose('z'), None);
    }

    #[test]
    fn generics_preserve_borrowing_defaults_and_classification_units() {
        let values = ['x'];
        let borrowed: Borrowed<'_> = Borrowed(&values);
        let named: Named<'_> = Named {
            predicate: borrowed.clone(),
        };
        let tuple: Tuple<'_> = Tuple(borrowed.clone());
        let expression: Expression<'_> = Expression(&values);
        let variants: [Generic<'_>; 3] = [
            Generic::Named {
                predicate: borrowed.clone(),
            },
            Generic::Tuple(borrowed.clone()),
            Generic::Expression { values: &values },
        ];

        check_choice(&borrowed);
        check_choice(&named);
        check_choice(&tuple);
        check_choice(&expression);
        for variant in &variants {
            check_choice(variant);
        }

        let values = ['x', 'y'];

        check_choice(&Named {
            predicate: Borrowed(&values),
        });
        check_choice(&Tuple(Borrowed(&values)));
        check_choice(&Expression(&values));
        check_choice(&Generic::Expression { values: &values });
    }

    /// Describes conjunction and disjunction boundaries only within negation's context.
    struct Events;

    impl Formatter for Events {
        fn enter<W>(writer: &mut W, segment: Segment, precedence: Precedence) -> fmt::Result
        where
            W: fmt::Write,
        {
            let text = match (precedence, segment) {
                (Precedence::NOT, Segment::Start(Connective::AND)) => "and[",
                (Precedence::NOT, Segment::Start(Connective::OR)) => "or[",
                (Precedence::NOT, Segment::Midway(BinaryConnective::AND)) => "&",
                (Precedence::NOT, Segment::Midway(BinaryConnective::OR)) => "|",
                (Precedence::NOT, Segment::End(Connective::AND | Connective::OR)) => "]",
                _ => return Err(fmt::Error),
            };

            writer.write_str(text)
        }
    }

    /// Field-free predicates governed by conjunction or disjunction identity.
    mod empty {
        use super::*;

        /// Field-free declarations share the selected connective's acceptance identity.
        macro_rules! shapes {
            ($module:ident, $policy:ident) => {
                /// Field-free predicates with the selected connective's acceptance identity.
                mod $module {
                    use super::*;

                    /// Its acceptance is the selected connective's identity.
                    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
                    #[assert($policy)]
                    pub struct Unit;

                    /// Its acceptance is the selected connective's identity.
                    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
                    #[assert($policy)]
                    pub struct Tuple();

                    /// Its acceptance is the selected connective's identity.
                    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
                    #[assert($policy)]
                    pub struct Named {}

                    /// Alternatives sharing the selected connective's acceptance identity.
                    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
                    pub enum Variants {
                        /// Accepts according to the connective's identity without operands.
                        #[assert($policy)]
                        Bare,
                        /// Accepts according to the connective's identity without operands.
                        #[assert($policy)]
                        Tuple(),
                        /// Accepts according to the connective's identity without operands.
                        #[assert($policy)]
                        Named {},
                    }
                }
            };
        }

        shapes!(and, conjunctive);
        shapes!(or, disjunctive);

        /// Requires the supplied acceptance identity, self-classification, and expected description.
        fn check<A>(value: A, identity: bool, expected: &str)
        where
            A: Choose<char, Unit = A> + PartialEq + fmt::Debug,
        {
            assert_eq!(value.assert('x'), identity);
            assert_eq!(value.choose('x').as_ref(), identity.then_some(&value));

            let mut output = String::new();
            assert_eq!(A::output_with::<_, Events>(&value, &mut output, Precedence::NOT), Ok(()));
            assert_eq!(output, expected);
        }

        #[test]
        fn all_empty_shapes_preserve_identities_and_balanced_events() {
            check(and::Unit, true, "and[]");
            check(and::Tuple(), true, "and[]");
            check(and::Named {}, true, "and[]");
            check(or::Unit, false, "or[]");
            check(or::Tuple(), false, "or[]");
            check(or::Named {}, false, "or[]");

            for value in [and::Variants::Bare, and::Variants::Tuple(), and::Variants::Named {}] {
                check(value, true, "and[]");
            }
            for value in [or::Variants::Bare, or::Variants::Tuple(), or::Variants::Named {}] {
                check(value, false, "or[]");
            }
        }

        /// Closing delimiters are an output failure for this writer.
        struct RejectEnd;

        impl fmt::Write for RejectEnd {
            fn write_str(&mut self, text: &str) -> fmt::Result {
                if text == "]" { Err(fmt::Error) } else { Ok(()) }
            }
        }

        #[test]
        fn empty_composites_propagate_closing_event_errors() {
            assert_eq!(
                <and::Unit as Assert<char>>::output_with::<_, Events>(&and::Unit, &mut RejectEnd, Precedence::NOT),
                Err(fmt::Error),
            );
            assert_eq!(
                <or::Variants as Assert<char>>::output_with::<_, Events>(&or::Variants::Tuple(), &mut RejectEnd, Precedence::NOT),
                Err(fmt::Error),
            );
        }
    }

    /// A predicate with an observable acceptance call and an independent label description.
    // NOTE(invariant): None.
    struct Probe<'a>(&'a RefCell<Vec<char>>, char, bool);

    impl Assert<char> for Probe<'_> {
        fn assert(&self, _: char) -> bool {
            let Self(log, label, result) = self;

            log.borrow_mut().push(*label);

            *result
        }

        fn output_with<W, F>(value: &Self, writer: &mut W, precedence: Precedence) -> fmt::Result
        where
            W: fmt::Write,
            F: Formatter,
        {
            let Self(_, label, _) = value;

            match precedence {
                Precedence::AND | Precedence::OR => writer.write_char(*label),
                _ => Err(fmt::Error),
            }
        }
    }

    /// Requires all operands to accept, with declaration-order predicate priority.
    #[derive(Assert)]
    #[assert(conjunctive)]
    // NOTE(invariant): None.
    struct All<A> {
        /// Highest-priority conjunct whose rejection is sufficient for failure.
        first: A,

        /// Additional requirement relevant when the first conjunct accepts.
        second: A,

        /// Final requirement for acceptance by the conjunction.
        third: A,
    }

    /// Accepts any operand, with declaration-order predicate priority.
    #[derive(Assert)]
    #[assert(disjunctive)]
    // NOTE(invariant): None.
    struct Any<A>(A, A, A);

    /// Alternatives selecting all-operand or any-operand acceptance.
    #[derive(Assert)]
    enum Ordered<A> {
        /// Requires all payload predicates to accept.
        #[assert(conjunctive)]
        All(A, A, A),
        /// Accepts any payload predicate in declaration-order priority.
        #[assert(disjunctive)]
        Any {
            /// Highest-priority alternative whose acceptance is sufficient.
            first: A,

            /// Alternative relevant when the first predicate rejects.
            second: A,

            /// Final alternative for acceptance by the disjunction.
            third: A,
        },
    }

    /// Requires observations `a` then `b`, the supplied acceptance result, and the expected description.
    fn check_order<A>(value: A, log: &RefCell<Vec<char>>, result: bool, expected: &str)
    where
        A: Assert<char>,
    {
        log.borrow_mut().clear();
        assert_eq!(value.assert('x'), result);
        assert_eq!(*log.borrow(), ['a', 'b']);

        let mut output = String::new();
        assert_eq!(A::output_with::<_, Events>(&value, &mut output, Precedence::NOT), Ok(()));
        assert_eq!(output, expected);
        assert_eq!(*log.borrow(), ['a', 'b']);
    }

    #[test]
    fn composites_short_circuit_in_field_order_but_format_every_field() {
        let log = RefCell::new(Vec::new());

        check_order(
            All {
                first: Probe(&log, 'a', true),
                second: Probe(&log, 'b', false),
                third: Probe(&log, 'c', true),
            },
            &log,
            false,
            "and[a&b&c]",
        );
        check_order(
            Any(Probe(&log, 'a', false), Probe(&log, 'b', true), Probe(&log, 'c', false)),
            &log,
            true,
            "or[a|b|c]",
        );
        check_order(
            Ordered::All(Probe(&log, 'a', true), Probe(&log, 'b', false), Probe(&log, 'c', true)),
            &log,
            false,
            "and[a&b&c]",
        );
        check_order(
            Ordered::Any {
                first: Probe(&log, 'a', false),
                second: Probe(&log, 'b', true),
                third: Probe(&log, 'c', false),
            },
            &log,
            true,
            "or[a|b|c]",
        );
    }
}
