//! Generic derives, empty shapes, and observable ordering.

#[cfg(test)]
mod tests {
    use core::{cell::RefCell, fmt};

    use aldebaran_logic::{
        fmt::{BinaryConnective, Connective, Formatter, Precedence, Segment},
        prelude::{Assert, Choose},
    };

    /// Borrowed predicate used to exercise `Choose` independently of `Assert` derive.
    #[derive(Debug, Clone, PartialEq, Eq, Choose)]
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

    /// Named composition retains lifetimes, defaults, and existing predicates.
    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
    #[assert(conjunctive)]
    struct Named<'a, T = char, const N: usize = 1>
    where
        T: Clone,
        T: PartialEq<char>,
    {
        /// Predicate borrowing the caller's array.
        predicate: Borrowed<'a, T, N>,
    }

    /// Tuple composition exercises the other structure expansion branch.
    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
    #[assert(disjunctive)]
    struct Tuple<'a, T = char, const N: usize = 1>(Borrowed<'a, T, N>)
    where
        T: Clone + PartialEq<char>;

    /// Expression assertions still retain generics not used by the expression.
    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
    #[assert(use 'x' type char)]
    struct Expression<'a, T = char, const N: usize = 1>(&'a [T; N])
    where
        T: Clone + PartialEq<char>;

    /// Enum expansion covers named, tuple, and expression assertions.
    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
    enum Generic<'a, T = char, const N: usize = 1>
    where
        T: Clone,
        T: PartialEq<char>,
    {
        /// Named conjunction over a borrowed predicate.
        #[assert(conjunctive)]
        Named {
            /// Predicate borrowing the caller's array.
            predicate: Borrowed<'a, T, N>,
        },
        /// Tuple disjunction over a borrowed predicate.
        #[assert(disjunctive)]
        Tuple(Borrowed<'a, T, N>),
        /// Expression constructed from a borrowed variant field.
        #[assert(use Borrowed(values) type Borrowed<'a, T, N>)]
        Expression {
            /// Caller-owned values used by the expression predicate.
            values: &'a [T; N],
        },
    }

    /// Classification must return the original assertion family, not its fields.
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

    /// Formatter exposing boundary kind and incoming precedence in its output.
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

    /// Empty shapes are intentional regression inputs, not API design examples.
    mod empty {
        use super::*;

        /// Define every empty structure and variant shape for a connective.
        macro_rules! shapes {
            ($module:ident, $policy:ident) => {
                mod $module {
                    use super::*;

                    /// Empty unit composition.
                    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
                    #[assert($policy)]
                    pub struct Unit;

                    /// Empty tuple composition.
                    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
                    #[assert($policy)]
                    pub struct Tuple();

                    /// Empty named composition.
                    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
                    #[assert($policy)]
                    pub struct Named {}

                    /// Empty variants retain their declared field shapes.
                    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
                    pub enum Variants {
                        /// Unit variant.
                        #[assert($policy)]
                        Bare,
                        /// Empty tuple variant.
                        #[assert($policy)]
                        Tuple(),
                        /// Empty named variant.
                        #[assert($policy)]
                        Named {},
                    }
                }
            };
        }

        shapes!(and, conjunctive);
        shapes!(or, disjunctive);

        /// Check the identity, classification unit, and balanced formatter events.
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

        /// Writer that accepts the opening event and rejects the closing event.
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

    /// Predicate recording evaluation order independently from formatting.
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

    /// Named conjunction evaluates fields in declaration order.
    #[derive(Assert)]
    #[assert(conjunctive)]
    struct All<A> {
        /// First predicate evaluated.
        first: A,

        /// Second predicate evaluated unless the first rejects.
        second: A,

        /// Last predicate evaluated unless an earlier predicate rejects.
        third: A,
    }

    /// Tuple disjunction evaluates fields in declaration order.
    #[derive(Assert)]
    #[assert(disjunctive)]
    struct Any<A>(A, A, A);

    /// Enum composite branches preserve the same ordering rules.
    #[derive(Assert)]
    enum Ordered<A> {
        /// Tuple conjunction.
        #[assert(conjunctive)]
        All(A, A, A),
        /// Named disjunction.
        #[assert(disjunctive)]
        Any {
            /// First predicate evaluated.
            first: A,

            /// Second predicate evaluated unless the first accepts.
            second: A,

            /// Last predicate evaluated unless an earlier predicate accepts.
            third: A,
        },
    }

    /// Formatting visits all fields while evaluation stops at a decisive field.
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
