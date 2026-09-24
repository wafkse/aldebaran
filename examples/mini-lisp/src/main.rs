//! A small Lisp lexer, parser, AST, evaluator, and diagnostic reporting example.

use core::fmt;

use aldebaran::{
    grammar::prelude::{Skipping, TokenStream},
    logic::{
        fmt::{Formatter, Precedence},
        prelude::{Assert, Choose, OneOf, Or},
    },
    prelude::{Span, Spanned},
    report::{
        codegen::Report,
        prelude::{Fancy, FancySettings, Fancyness, InlineAnnotations, Label, Present, RenderMut, Report, Textual},
    },
    source::prelude::SourceDissect,
    text::{
        basic::{Ascii, punct::Parenthesis},
        prelude::{
            AsciiAlphabetic, AsciiAlphanumeric, AsciiDigit, AsciiWhitespace, Internment, Lex, LexStream, OwnedStorage, StorageId, Text,
        },
    },
};

/// Symbol punctuation shared by the start and continuation predicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert)]
#[assert(disjunctive)]
struct SymbolPunctuation(Ascii, Ascii, Ascii, Ascii, Ascii, Ascii, Ascii, Ascii, Ascii, Ascii);

/// Every punctuation character admitted in a Lisp symbol.
const SYMBOL_PUNCTUATION: SymbolPunctuation = SymbolPunctuation(
    Ascii::Underscore,
    Ascii::Plus,
    Ascii::Minus,
    Ascii::Asterisk,
    Ascii::Slash,
    Ascii::Equals,
    Ascii::LessThan,
    Ascii::GreaterThan,
    Ascii::Question,
    Ascii::Exclamation,
);

/// Predicate for the first source character of a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert)]
#[assert(use Or::one(AsciiAlphabetic, SYMBOL_PUNCTUATION) type Or<AsciiAlphabetic, SymbolPunctuation>)]
struct SymbolStart;

/// Predicate for source characters after the first symbol character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert)]
#[assert(use Or::one(AsciiAlphanumeric, SYMBOL_PUNCTUATION) type Or<AsciiAlphanumeric, SymbolPunctuation>)]
struct SymbolRest;

/// Semantic source fragment expectations used by lexical recognition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Assert, Choose)]
enum Fragment {
    /// Whitespace trivia.
    #[assert(use AsciiWhitespace type AsciiWhitespace)]
    Trivia,

    /// Decimal integer spelling.
    #[assert(use AsciiDigit type AsciiDigit)]
    Integer,

    /// First character of an interned symbol.
    #[assert(use SymbolStart type SymbolStart)]
    Symbol,

    /// Opening list delimiter.
    #[assert(use Parenthesis::Left type Parenthesis)]
    LeftParenthesis,

    /// Closing list delimiter.
    #[assert(use Parenthesis::Right type Parenthesis)]
    RightParenthesis,
}

/// Complete lexical choice used to classify the next source character.
const FRAGMENTS: OneOf<Fragment, 5> = OneOf::these([
    Fragment::Trivia,
    Fragment::Integer,
    Fragment::Symbol,
    Fragment::LeftParenthesis,
    Fragment::RightParenthesis,
]);

/// Semantic token produced by [`Lex`] and consumed directly by the parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Token {
    /// Parser-invisible whitespace.
    Trivia(Span),

    /// Parsed integer value and its source extent.
    Integer(i64, Span),

    /// Interned symbol identity and its source extent.
    Symbol(StorageId, Span),

    /// Opening list delimiter.
    LeftParenthesis(Span),

    /// Closing list delimiter.
    RightParenthesis(Span),
}

impl Token {
    /// Determine whether this token should be hidden from the parser.
    #[inline]
    const fn is_trivia(&self) -> bool {
        matches!(self, Self::Trivia(_))
    }
}

impl Spanned for Token {
    #[inline]
    fn span(&self) -> Span {
        match *self {
            Self::Trivia(span)
            | Self::Integer(_, span)
            | Self::Symbol(_, span)
            | Self::LeftParenthesis(span)
            | Self::RightParenthesis(span) => span,
        }
    }
}

/// Grammar-level token predicates used by recursive descent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Form {
    /// Any token that may begin an expression.
    Expression,

    /// Closing delimiter for the current list.
    ListEnd,
}

impl Form {
    /// Human-facing predicate description.
    #[inline]
    const fn description(&self) -> &'static str {
        match self {
            Self::Expression => "expression",
            Self::ListEnd => "`)`",
        }
    }
}

impl Assert<Token> for Form {
    #[inline]
    fn assert(&self, input: Token) -> bool {
        match self {
            Self::Expression => matches!(input, Token::Integer(..) | Token::Symbol(..) | Token::LeftParenthesis(_)),
            Self::ListEnd => matches!(input, Token::RightParenthesis(_)),
        }
    }

    #[inline]
    fn output_with<W, F>(target_value: &Self, writer: &mut W, _: Precedence) -> fmt::Result
    where
        W: fmt::Write,
        F: Formatter,
    {
        writer.write_str(target_value.description())
    }
}

/// Report for a source character rejected by every lexical fragment.
#[derive(Debug, Report)]
#[error("unexpected character")]
#[report(title = "unexpected character", message = "expected whitespace, an integer, a symbol, `(`, or `)`")]
struct UnexpectedCharacter(Span);

/// Report for a decimal integer outside the supported range.
#[derive(Debug, Report)]
#[error("integer is out of range")]
#[report(title = "integer is out of range", message = "this integer does not fit in a signed 64 bit value")]
struct IntegerOutOfRange(Span);

/// Report for exhausted symbol identity storage.
#[derive(Debug, Report)]
#[error("symbol storage is exhausted")]
#[report(
    title = "symbol storage is exhausted",
    message = "this symbol could not be assigned an interned identity"
)]
struct SymbolStorageExhausted(Span);

/// Every failure that can occur while lexing.
#[derive(Debug, Report)]
enum LexError {
    /// No lexical fragment accepts the next source character.
    #[error(transparent(0))]
    #[report(transparent)]
    UnexpectedCharacter(UnexpectedCharacter),

    /// Integer conversion failure.
    #[error(transparent(0))]
    #[report(transparent)]
    IntegerOutOfRange(IntegerOutOfRange),

    /// Internment capacity failure.
    #[error(transparent(0))]
    #[report(transparent)]
    SymbolStorageExhausted(SymbolStorageExhausted),
}

impl<'source> Lex<'source, str> for Token {
    type Input<'input>
        = (Fragment, &'source str, Span)
    where
        'source: 'input;

    type Context = OwnedStorage<str>;

    type Error = LexError;

    fn pass<'input>(text: &'input mut Text<'source, str>) -> Result<Self::Input<'input>, Self::Error>
    where
        'source: 'input,
    {
        let index = text.index();
        let target_source = text.source();
        let (fragment, first) = text
            .expected_is_next_spanned(FRAGMENTS)
            .map_err(|_| LexError::UnexpectedCharacter(UnexpectedCharacter(Span::unit(index))))?;
        let rest = match fragment {
            Fragment::Trivia | Fragment::Integer => text.ignore_while_spanned(fragment),
            Fragment::Symbol => text.ignore_while_spanned(SymbolRest),
            Fragment::LeftParenthesis | Fragment::RightParenthesis => None,
        };
        let span = rest.map_or(first, |rest| first.superset(rest));
        let spelling = target_source.dissect(span);

        Ok((fragment, spelling, span))
    }

    fn build_with_ctx<'input>((fragment, spelling, span): Self::Input<'input>, context: &mut Self::Context) -> Result<Self, Self::Error>
    where
        'source: 'input,
    {
        match fragment {
            Fragment::Trivia => Ok(Self::Trivia(span)),
            Fragment::Integer => {
                let value = spelling
                    .parse::<i64>()
                    .map_err(|_| LexError::IntegerOutOfRange(IntegerOutOfRange(span)))?;

                Ok(Self::Integer(value, span))
            }
            Fragment::Symbol => {
                let symbol = context
                    .try_store(spelling)
                    .ok_or(LexError::SymbolStorageExhausted(SymbolStorageExhausted(span)))?;

                Ok(Self::Symbol(symbol, span))
            }
            Fragment::LeftParenthesis => Ok(Self::LeftParenthesis(span)),
            Fragment::RightParenthesis => Ok(Self::RightParenthesis(span)),
        }
    }
}

/// One parsed Lisp program.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Program(Vec<Expr>);

impl Program {
    /// Borrow the top-level expressions in source order.
    #[inline]
    fn expressions(&self) -> &[Expr] {
        let Self(expressions) = self;

        expressions
    }

    /// Parse every top-level expression from one parser-visible token stream.
    fn parse<S>(stream: &mut S, eof: Span) -> Result<Self, FrontendError>
    where
        S: TokenStream<Token = Token, Error = LexError> + ?Sized,
    {
        let mut expressions = Vec::new();

        loop {
            match TokenStream::next(stream).map_err(FrontendError::Lex)? {
                Some(token) => match Form::ListEnd.assert(token) {
                    true => {
                        let error = UnexpectedClosingParenthesis(token.span());

                        break Err(FrontendError::Parse(ParseError::UnexpectedClosingParenthesis(error)));
                    }
                    false => expressions.push(Expr::parse_from(stream, token, eof)?),
                },
                None => break Ok(Self(expressions)),
            }
        }
    }
}

/// AST expression retaining source context for later diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Expr {
    /// Expression representation.
    kind: ExprKind,

    /// Complete source extent for this expression.
    span: Span,
}

impl Expr {
    /// Borrow the expression representation.
    #[inline]
    const fn kind(&self) -> &ExprKind {
        let Self { kind, .. } = self;

        kind
    }

    /// Retrieve the complete source extent.
    #[inline]
    const fn span(&self) -> Span {
        let Self { span, .. } = self;

        *span
    }

    /// Parse one expression after its first token has already been consumed.
    fn parse_from<S>(stream: &mut S, token: Token, eof: Span) -> Result<Self, FrontendError>
    where
        S: TokenStream<Token = Token, Error = LexError> + ?Sized,
    {
        match Form::Expression.assert(token) {
            true => match token {
                Token::Integer(value, span) => {
                    let kind = ExprKind::Integer(value);

                    Ok(Self { kind, span })
                }
                Token::Symbol(symbol, span) => {
                    let kind = ExprKind::Symbol(symbol);

                    Ok(Self { kind, span })
                }
                Token::LeftParenthesis(span) => Self::parse_list(stream, span, eof),
                Token::Trivia(_) | Token::RightParenthesis(_) => {
                    let error = UnexpectedToken(token.span());

                    Err(FrontendError::Parse(ParseError::UnexpectedToken(error)))
                }
            },
            false => {
                let error = UnexpectedToken(token.span());

                Err(FrontendError::Parse(ParseError::UnexpectedToken(error)))
            }
        }
    }

    /// Parse one list directly from the remaining token stream.
    fn parse_list<S>(stream: &mut S, opening: Span, eof: Span) -> Result<Self, FrontendError>
    where
        S: TokenStream<Token = Token, Error = LexError> + ?Sized,
    {
        let mut elements = Vec::new();

        loop {
            match TokenStream::next(stream).map_err(FrontendError::Lex)? {
                Some(token) => {
                    let closes_list = Form::ListEnd.assert(token);
                    let starts_expression = Form::Expression.assert(token);

                    match (closes_list, starts_expression) {
                        (true, false) => {
                            let span = opening.superset(token.span());
                            let kind = ExprKind::List(elements);

                            break Ok(Self { kind, span });
                        }
                        (false, true) => elements.push(Self::parse_from(stream, token, eof)?),
                        _ => {
                            let error = UnexpectedToken(token.span());

                            break Err(FrontendError::Parse(ParseError::UnexpectedToken(error)));
                        }
                    }
                }
                None => {
                    let error = UnclosedList::new(opening, eof);

                    break Err(FrontendError::Parse(ParseError::UnclosedList(error)));
                }
            }
        }
    }
}

/// AST expression representation.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ExprKind {
    /// Literal integer.
    Integer(i64),

    /// Interned symbol.
    Symbol(StorageId),

    /// Parenthesized Lisp form.
    List(Vec<Expr>),
}

/// Report for a token that cannot appear in the current parser position.
#[derive(Debug, Report)]
#[error("unexpected token")]
#[report(title = "unexpected token", message = "expected an expression or a closing parenthesis")]
struct UnexpectedToken(Span);

/// Report for an unmatched closing parenthesis.
#[derive(Debug, Report)]
#[error("unexpected closing parenthesis")]
#[report(title = "unexpected closing parenthesis", message = "there is no open list for this delimiter")]
struct UnexpectedClosingParenthesis(Span);

/// Report for source exhaustion while a list remains open.
#[derive(Debug, Report)]
#[error("unclosed list")]
#[report(title = "unclosed list", annotations = annotations)]
struct UnclosedList {
    /// Primary opening delimiter and related end boundary.
    annotations: InlineAnnotations<Label<&'static str>, 1>,
}

impl UnclosedList {
    /// Build a diagnostic connecting the missing delimiter to its opening site.
    #[inline]
    const fn new(opening: Span, eof: Span) -> Self {
        let primary = Label::new("this list starts here and is never closed", opening);
        let related = Label::new("expected `)` before the end of input", eof);
        let annotations = InlineAnnotations::new(primary, [related]);

        Self { annotations }
    }
}

/// Every syntactic failure produced after lexical tokens are available.
#[derive(Debug, Report)]
enum ParseError {
    /// Token violates a parser predicate.
    #[error(transparent(0))]
    #[report(transparent)]
    UnexpectedToken(UnexpectedToken),

    /// Unmatched closing delimiter.
    #[error(transparent(0))]
    #[report(transparent)]
    UnexpectedClosingParenthesis(UnexpectedClosingParenthesis),

    /// Missing closing delimiter.
    #[error(transparent(0))]
    #[report(transparent)]
    UnclosedList(UnclosedList),
}

/// Front-end failure preserving whether lexing or parsing rejected the source.
#[derive(Debug, Report)]
enum FrontendError {
    /// Lazy lexical stream failure.
    #[error(transparent(0))]
    #[report(transparent)]
    Lex(LexError),

    /// Syntactic failure.
    #[error(transparent(0))]
    #[report(transparent)]
    Parse(ParseError),
}

/// Runtime value produced by evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Value {
    /// Integer value.
    Integer(i64),

    /// Boolean value.
    Boolean(bool),

    /// Value of an empty program.
    Nil,
}

impl Value {
    /// Require an integer value while retaining source context on failure.
    #[inline]
    fn into_integer(self, value: Span, call: Span) -> Result<i64, EvalError> {
        match self {
            Self::Integer(value) => Ok(value),
            Self::Boolean(_) | Self::Nil => Err(EvalError::ExpectedInteger(ExpectedInteger::new(value, call))),
        }
    }

    /// Require a boolean value while retaining source context on failure.
    #[inline]
    fn into_boolean(self, value: Span, call: Span) -> Result<bool, EvalError> {
        match self {
            Self::Boolean(value) => Ok(value),
            Self::Integer(_) | Self::Nil => Err(EvalError::ExpectedBoolean(ExpectedBoolean::new(value, call))),
        }
    }
}

/// Built-in forms recognized in operator position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operator {
    /// Bind a symbol.
    Define,

    /// Conditional evaluation.
    If,

    /// Integer addition.
    Add,

    /// Integer subtraction.
    Subtract,

    /// Integer multiplication.
    Multiply,

    /// Integer division.
    Divide,

    /// Integer equality.
    Equal,
}

impl Operator {
    /// Classify one interned operator spelling.
    #[inline]
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "define" => Some(Self::Define),
            "if" => Some(Self::If),
            "+" => Some(Self::Add),
            "-" => Some(Self::Subtract),
            "*" => Some(Self::Multiply),
            "/" => Some(Self::Divide),
            "=" => Some(Self::Equal),
            _ => None,
        }
    }
}

/// Report for an empty list used as an expression.
#[derive(Debug, Report)]
#[error("empty list is not callable")]
#[report(
    title = "empty list is not callable",
    message = "a Lisp form needs an operator in its first position"
)]
struct EmptyList(Span);

/// Report for a non-symbol expression used as an operator.
#[derive(Debug, Report)]
#[error("expected an operator")]
#[report(title = "expected an operator", annotations = annotations)]
struct ExpectedOperator {
    /// Primary operator annotation and containing call annotation.
    annotations: InlineAnnotations<Label<&'static str>, 1>,
}

impl ExpectedOperator {
    /// Build an operator-position diagnostic.
    #[inline]
    const fn new(operator: Span, call: Span) -> Self {
        let primary = Label::new("this expression is not a symbol", operator);
        let related = Label::new("while evaluating this call", call);
        let annotations = InlineAnnotations::new(primary, [related]);

        Self { annotations }
    }
}

/// Report for an unresolved variable or operator symbol.
#[derive(Debug, Report)]
#[error("unbound symbol")]
#[report(title = "unbound symbol", message = "this symbol has no binding or built-in meaning")]
struct UnboundSymbol(Span);

/// Report for a form receiving the wrong number of arguments.
#[derive(Debug, Report)]
#[error("wrong number of arguments")]
#[report(title = "wrong number of arguments", annotations = annotations)]
struct WrongArity {
    /// Expected argument count retained as structured error state.
    #[allow(dead_code)] // Retained for programmatic error inspection beyond this example.
    expected: usize,

    /// Actual argument count retained as structured error state.
    #[allow(dead_code)] // Retained for programmatic error inspection beyond this example.
    actual: usize,

    /// Primary call annotation and operator annotation.
    annotations: InlineAnnotations<Label<&'static str>, 1>,
}

impl WrongArity {
    /// Build an arity diagnostic while preserving numeric error identity.
    #[inline]
    const fn new(expected: usize, actual: usize, operator: Span, call: Span) -> Self {
        let primary = Label::new("this call has the wrong number of arguments", call);
        let related = Label::new("the selected operator has a fixed arity", operator);
        let annotations = InlineAnnotations::new(primary, [related]);

        Self {
            expected,
            actual,
            annotations,
        }
    }
}

/// Report for a non-symbol binding name.
#[derive(Debug, Report)]
#[error("expected a binding name")]
#[report(title = "expected a binding name", annotations = annotations)]
struct ExpectedBindingName {
    /// Primary name annotation and containing definition annotation.
    annotations: InlineAnnotations<Label<&'static str>, 1>,
}

impl ExpectedBindingName {
    /// Build a definition-name diagnostic.
    #[inline]
    const fn new(name: Span, call: Span) -> Self {
        let primary = Label::new("this expression cannot name a binding", name);
        let related = Label::new("while evaluating this definition", call);
        let annotations = InlineAnnotations::new(primary, [related]);

        Self { annotations }
    }
}

/// Report for a non-integer arithmetic operand.
#[derive(Debug, Report)]
#[error("expected an integer")]
#[report(title = "expected an integer", annotations = annotations)]
struct ExpectedInteger {
    /// Primary value annotation and containing call annotation.
    annotations: InlineAnnotations<Label<&'static str>, 1>,
}

impl ExpectedInteger {
    /// Build an integer type diagnostic.
    #[inline]
    const fn new(value: Span, call: Span) -> Self {
        let primary = Label::new("this expression did not evaluate to an integer", value);
        let related = Label::new("an integer is required by this call", call);
        let annotations = InlineAnnotations::new(primary, [related]);

        Self { annotations }
    }
}

/// Report for a non-boolean conditional test.
#[derive(Debug, Report)]
#[error("expected a boolean")]
#[report(title = "expected a boolean", annotations = annotations)]
struct ExpectedBoolean {
    /// Primary value annotation and containing conditional annotation.
    annotations: InlineAnnotations<Label<&'static str>, 1>,
}

impl ExpectedBoolean {
    /// Build a conditional type diagnostic.
    #[inline]
    const fn new(value: Span, call: Span) -> Self {
        let primary = Label::new("this condition did not evaluate to a boolean", value);
        let related = Label::new("while selecting a branch in this conditional", call);
        let annotations = InlineAnnotations::new(primary, [related]);

        Self { annotations }
    }
}

/// Report for integer division by zero.
#[derive(Debug, Report)]
#[error("division by zero")]
#[report(title = "division by zero", annotations = annotations)]
struct DivisionByZero {
    /// Primary divisor annotation and containing call annotation.
    annotations: InlineAnnotations<Label<&'static str>, 1>,
}

impl DivisionByZero {
    /// Build a division-by-zero diagnostic.
    #[inline]
    const fn new(divisor: Span, call: Span) -> Self {
        let primary = Label::new("this divisor evaluates to zero", divisor);
        let related = Label::new("while evaluating this division", call);
        let annotations = InlineAnnotations::new(primary, [related]);

        Self { annotations }
    }
}

/// Report for a checked arithmetic operation that overflows.
#[derive(Debug, Report)]
#[error("integer arithmetic overflow")]
#[report(
    title = "integer arithmetic overflow",
    message = "this operation cannot be represented as a signed 64 bit integer"
)]
struct ArithmeticOverflow(Span);

/// Every failure that can occur during evaluation.
#[derive(Debug, Report)]
enum EvalError {
    /// Empty call form.
    #[error(transparent(0))]
    #[report(transparent)]
    EmptyList(EmptyList),

    /// Invalid operator position.
    #[error(transparent(0))]
    #[report(transparent)]
    ExpectedOperator(ExpectedOperator),

    /// Unknown variable or form.
    #[error(transparent(0))]
    #[report(transparent)]
    UnboundSymbol(UnboundSymbol),

    /// Fixed arity violation.
    #[error(transparent(0))]
    #[report(transparent)]
    WrongArity(WrongArity),

    /// Invalid definition target.
    #[error(transparent(0))]
    #[report(transparent)]
    ExpectedBindingName(ExpectedBindingName),

    /// Integer type mismatch.
    #[error(transparent(0))]
    #[report(transparent)]
    ExpectedInteger(ExpectedInteger),

    /// Boolean type mismatch.
    #[error(transparent(0))]
    #[report(transparent)]
    ExpectedBoolean(ExpectedBoolean),

    /// Zero divisor.
    #[error(transparent(0))]
    #[report(transparent)]
    DivisionByZero(DivisionByZero),

    /// Checked integer overflow.
    #[error(transparent(0))]
    #[report(transparent)]
    ArithmeticOverflow(ArithmeticOverflow),
}

/// Evaluator state containing symbol storage and lexical bindings.
#[derive(Debug)]
struct Evaluator<'storage> {
    /// Symbol spelling storage created during lexing.
    storage: &'storage OwnedStorage<str>,

    /// Current bindings keyed by interned symbol identity.
    bindings: Vec<(StorageId, Value)>,
}

impl<'storage> Evaluator<'storage> {
    /// Create an evaluator over one symbol identity domain.
    #[inline]
    fn new(storage: &'storage OwnedStorage<str>) -> Self {
        let bindings = Vec::new();

        Self { storage, bindings }
    }

    /// Evaluate every top-level expression and return the final value.
    fn evaluate_program(&mut self, program: &Program) -> Result<Value, EvalError> {
        let mut result = Value::Nil;

        for expression in program.expressions() {
            result = self.evaluate(expression)?;
        }

        Ok(result)
    }

    /// Evaluate one AST expression.
    fn evaluate(&mut self, expression: &Expr) -> Result<Value, EvalError> {
        match expression.kind() {
            ExprKind::Integer(value) => Ok(Value::Integer(*value)),
            ExprKind::Symbol(symbol) => self
                .lookup(*symbol)
                .ok_or_else(|| EvalError::UnboundSymbol(UnboundSymbol(expression.span()))),
            ExprKind::List(elements) => self.evaluate_list(expression.span(), elements),
        }
    }

    /// Resolve one lexical binding.
    #[inline]
    fn lookup(&self, symbol: StorageId) -> Option<Value> {
        let Self { bindings, .. } = self;

        bindings
            .iter()
            .rev()
            .find_map(|(bound_symbol, value)| (*bound_symbol == symbol).then_some(*value))
    }

    /// Bind or replace one symbol value.
    fn bind(&mut self, symbol: StorageId, value: Value) {
        let Self { bindings, .. } = self;
        let existing = bindings.iter_mut().rev().find(|(bound_symbol, _)| *bound_symbol == symbol);

        match existing {
            Some((_, slot)) => *slot = value,
            None => bindings.push((symbol, value)),
        }
    }

    /// Classify an operator symbol through the shared internment storage.
    #[inline]
    fn operator(&self, symbol: StorageId) -> Option<Operator> {
        let Self { storage, .. } = self;

        storage.try_resolve(symbol).and_then(Operator::from_name)
    }

    /// Evaluate one call form.
    fn evaluate_list(&mut self, call: Span, elements: &[Expr]) -> Result<Value, EvalError> {
        match elements.split_first() {
            None => Err(EvalError::EmptyList(EmptyList(call))),
            Some((operator_expression, arguments)) => {
                let symbol = match operator_expression.kind() {
                    ExprKind::Symbol(symbol) => Ok(*symbol),
                    ExprKind::Integer(_) | ExprKind::List(_) => {
                        let operator = operator_expression.span();

                        Err(EvalError::ExpectedOperator(ExpectedOperator::new(operator, call)))
                    }
                }?;
                let operator = self
                    .operator(symbol)
                    .ok_or_else(|| EvalError::UnboundSymbol(UnboundSymbol(operator_expression.span())))?;

                self.evaluate_operator(operator, operator_expression.span(), call, arguments)
            }
        }
    }

    /// Dispatch one built-in operator.
    fn evaluate_operator(&mut self, operator: Operator, operator_span: Span, call: Span, arguments: &[Expr]) -> Result<Value, EvalError> {
        match operator {
            Operator::Define => self.evaluate_define(operator_span, call, arguments),
            Operator::If => self.evaluate_if(operator_span, call, arguments),
            Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide | Operator::Equal => {
                self.evaluate_binary(operator, operator_span, call, arguments)
            }
        }
    }

    /// Evaluate one definition form.
    fn evaluate_define(&mut self, operator: Span, call: Span, arguments: &[Expr]) -> Result<Value, EvalError> {
        match arguments {
            [name, value] => {
                let symbol = match name.kind() {
                    ExprKind::Symbol(symbol) => Ok(*symbol),
                    ExprKind::Integer(_) | ExprKind::List(_) => {
                        Err(EvalError::ExpectedBindingName(ExpectedBindingName::new(name.span(), call)))
                    }
                }?;
                let value = self.evaluate(value)?;

                self.bind(symbol, value);

                Ok(value)
            }
            _ => Err(EvalError::WrongArity(WrongArity::new(2, arguments.len(), operator, call))),
        }
    }

    /// Evaluate one conditional form with lazy branch selection.
    fn evaluate_if(&mut self, operator: Span, call: Span, arguments: &[Expr]) -> Result<Value, EvalError> {
        match arguments {
            [condition, then_expression, else_expression] => {
                let value = self.evaluate(condition)?;
                let condition_value = value.into_boolean(condition.span(), call)?;

                match condition_value {
                    true => self.evaluate(then_expression),
                    false => self.evaluate(else_expression),
                }
            }
            _ => Err(EvalError::WrongArity(WrongArity::new(3, arguments.len(), operator, call))),
        }
    }

    /// Evaluate one fixed-arity integer or comparison form.
    fn evaluate_binary(&mut self, operator: Operator, operator_span: Span, call: Span, arguments: &[Expr]) -> Result<Value, EvalError> {
        match arguments {
            [left, right] => {
                let left_value = self.evaluate(left)?.into_integer(left.span(), call)?;
                let right_value = self.evaluate(right)?.into_integer(right.span(), call)?;

                match operator {
                    Operator::Add => left_value
                        .checked_add(right_value)
                        .map(Value::Integer)
                        .ok_or(EvalError::ArithmeticOverflow(ArithmeticOverflow(call))),
                    Operator::Subtract => left_value
                        .checked_sub(right_value)
                        .map(Value::Integer)
                        .ok_or(EvalError::ArithmeticOverflow(ArithmeticOverflow(call))),
                    Operator::Multiply => left_value
                        .checked_mul(right_value)
                        .map(Value::Integer)
                        .ok_or(EvalError::ArithmeticOverflow(ArithmeticOverflow(call))),
                    Operator::Divide => match right_value {
                        0 => Err(EvalError::DivisionByZero(DivisionByZero::new(right.span(), call))),
                        _ => left_value
                            .checked_div(right_value)
                            .map(Value::Integer)
                            .ok_or(EvalError::ArithmeticOverflow(ArithmeticOverflow(call))),
                    },
                    Operator::Equal => Ok(Value::Boolean(left_value == right_value)),
                    Operator::Define | Operator::If => Err(EvalError::UnboundSymbol(UnboundSymbol(operator_span))),
                }
            }
            _ => Err(EvalError::WrongArity(WrongArity::new(2, arguments.len(), operator_span, call))),
        }
    }
}

/// Top-level language failure preserving front-end and evaluation identity.
#[derive(Debug, Report)]
enum LispError {
    /// Lexing or parsing failure.
    #[error(transparent(0))]
    #[report(transparent)]
    Frontend(FrontendError),

    /// Evaluation failure.
    #[error(transparent(0))]
    #[report(transparent)]
    Eval(EvalError),
}

/// Parse and evaluate one source program through the live lexical stream.
fn execute(source: &str) -> Result<(Program, Value), LispError> {
    let mut storage = OwnedStorage::<str>::empty();
    let eof = Span::unit(source.len());
    let program = {
        let internment = Internment::new(source, &mut storage);
        let lexed = LexStream::<str, Token>::new(internment);
        let mut visible = Skipping::new(lexed, Token::is_trivia);

        Program::parse(&mut visible, eof).map_err(LispError::Frontend)?
    };
    let mut evaluator = Evaluator::new(&storage);
    let value = evaluator.evaluate_program(&program).map_err(LispError::Eval)?;

    Ok((program, value))
}

/// Render one language report against its original source.
fn render_error(source: &str, error: &LispError) -> String {
    let attached = Report::attach(error, source);
    let mut output = String::new();
    let mut renderer = Textual::<'_, _, Fancy, _>::new(&mut output);
    let settings = Present::from_input(FancySettings::level(Fancyness::Outblown));

    renderer
        .render_mut_with_input(settings, &attached)
        .expect("rendering a report into a String succeeds");

    output
}

/// Run one source and print either its AST and result or its diagnostic.
fn demonstrate(name: &str, source: &str) {
    println!("\n== {name} ==\n{source}");

    match execute(source) {
        Ok((program, value)) => {
            println!("AST\n{program:#?}");
            println!("result {value:?}");
        }
        Err(error) => println!("{}", render_error(source, &error)),
    }
}

fn main() {
    let valid = "(define radius 6)\n(define area (* radius radius))\n(if (= area 36) (+ area 6) 0)\n";
    let type_error = "(define answer 42)\n(+ answer (= answer 42))\n";
    let parse_error = "(+ 1 (* 2 3)\n";
    let lex_error = "(+ 1 @)\n";

    demonstrate("valid program", valid);
    demonstrate("typed evaluation error", type_error);
    demonstrate("parse error", parse_error);
    demonstrate("lex error", lex_error);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Require one report to identify at least one source extent.
    fn assert_has_target<R>(report: &R)
    where
        R: Report,
    {
        assert!(Report::target(report).is_some());
    }

    #[test]
    fn every_leaf_report_targets_source() {
        let span = Span::MIN;

        assert_has_target(&UnexpectedCharacter(span));
        assert_has_target(&IntegerOutOfRange(span));
        assert_has_target(&SymbolStorageExhausted(span));
        assert_has_target(&UnexpectedToken(span));
        assert_has_target(&UnexpectedClosingParenthesis(span));
        assert_has_target(&UnclosedList::new(span, span));
        assert_has_target(&EmptyList(span));
        assert_has_target(&ExpectedOperator::new(span, span));
        assert_has_target(&UnboundSymbol(span));
        assert_has_target(&WrongArity::new(2, 1, span, span));
        assert_has_target(&ExpectedBindingName::new(span, span));
        assert_has_target(&ExpectedInteger::new(span, span));
        assert_has_target(&ExpectedBoolean::new(span, span));
        assert_has_target(&DivisionByZero::new(span, span));
        assert_has_target(&ArithmeticOverflow(span));
    }
}
