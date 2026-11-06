//! Assertion evaluation order, empty identities, and formatting behavior.

#[cfg(test)]
mod tests {
    use core::{cell::RefCell, fmt};

    use aldebaran_logic::{
        fmt::{BinaryConnective, Connective, Formatter, Precedence, Segment},
        prelude::{Assert, Choose},
    };

    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
    enum PatternToken {
        Digit(u8),
        End,
    }

    #[derive(Assert)]
    #[aldebaran(crate)]
    #[assert(input(PatternToken))]
    enum PatternExpectation {
        #[assert(matches(PatternToken::Digit(0..=9)) as "digit")]
        Digit,

        #[assert(matches(PatternToken::End) as "end")]
        End,

        #[cfg(target_os = "windows")]
        #[assert(matches(PatternToken::Digit(10..=u8::MAX)) as "wide digit")]
        WideDigit,

        #[cfg_attr(target_os = "linux", cfg(windows))]
        #[assert(matches(PatternToken::End) as "conditional end")]
        ConditionalEnd,
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

    #[test]
    fn structural_expectations_match_inputs_and_describe_the_selected_variant() {
        assert!(PatternExpectation::Digit.assert(PatternToken::Digit(4)));
        assert!(!PatternExpectation::Digit.assert(PatternToken::Digit(12)));
        assert!(!PatternExpectation::Digit.assert(PatternToken::End));

        let mut output = String::new();
        assert_eq!(
            <PatternExpectation as Assert<PatternToken>>::output_with::<_, Events>(&PatternExpectation::End, &mut output, Precedence::NOT,),
            Ok(()),
            "structural expectation formatting should succeed",
        );
        assert_eq!(output, "end", "formatting should describe the selected variant");
    }

    /// Field-free predicates governed by conjunction or disjunction identity.
    mod empty {
        use super::*;

        /// Field-free declarations share the selected connective's acceptance identity.
        macro_rules! shape {
            ($module:ident, $policy:ident) => {
                /// Field-free predicates with the selected connective's acceptance identity.
                mod $module {
                    use super::*;

                    /// Its acceptance is the selected connective's identity.
                    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
                    #[aldebaran(crate)]
                    #[assert($policy)]
                    pub struct Unit;

                    /// Its acceptance is the selected connective's identity.
                    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
                    #[aldebaran(crate)]
                    #[assert($policy)]
                    pub struct Tuple();

                    /// Its acceptance is the selected connective's identity.
                    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
                    #[aldebaran(crate)]
                    #[assert($policy)]
                    pub struct Named {}

                    /// Alternatives sharing the selected connective's acceptance identity.
                    #[derive(Debug, Clone, PartialEq, Eq, Assert, Choose)]
                    #[aldebaran(crate)]
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

        shape!(and, conjunctive);
        shape!(or, disjunctive);

        /// Requires the supplied acceptance identity, self-classification, and expected description.
        fn check<A>(value: A, identity: bool, expected: &str)
        where
            A: Choose<char, Unit = A> + PartialEq + fmt::Debug,
        {
            assert_eq!(
                value.assert('x'),
                identity,
                "predicate result must follow the empty connective identity"
            );
            assert_eq!(
                value.choose('x').as_ref(),
                identity.then_some(&value),
                "derived classification must follow the predicate result",
            );

            let mut output = String::new();
            assert_eq!(
                A::output_with::<_, Events>(&value, &mut output, Precedence::NOT),
                Ok(()),
                "formatting an empty connective must succeed",
            );
            assert_eq!(output, expected, "empty connective formatting must be balanced");
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
    #[aldebaran(crate)]
    #[assert(conjunctive)]
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
    #[aldebaran(crate)]
    #[assert(disjunctive)]
    struct Any<A>(A, A, A);

    /// Alternatives selecting all-operand or any-operand acceptance.
    #[derive(Assert)]
    #[aldebaran(crate)]
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
        assert_eq!(value.assert('x'), result, "composite acceptance must follow its operands");
        assert_eq!(*log.borrow(), ['a', 'b'], "acceptance must short-circuit in field order");

        let mut output = String::new();
        assert_eq!(
            A::output_with::<_, Events>(&value, &mut output, Precedence::NOT),
            Ok(()),
            "formatting a composite must succeed",
        );
        assert_eq!(output, expected, "composite formatting must include every operand");
        assert_eq!(*log.borrow(), ['a', 'b'], "formatting must not evaluate predicates");
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
