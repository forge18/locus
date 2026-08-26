//! A small, deterministic condition language for workflow routing.
//!
//! Conditions are compiled into a typed expression tree and evaluated only against a caller-owned
//! [`RunSnapshot`].  The parser deliberately has no access to the store, filesystem, network, or
//! model runtime: a condition is a `WHERE`-style predicate over facts already captured for a run.

use std::fmt;

pub use super::workflow::ConditionOperand;

const MAX_EXPRESSION_LENGTH: usize = 4_096;
const MAX_TOKENS: usize = 1_024;
const MAX_NESTING: usize = 64;
const MAX_SNAPSHOT_EVENTS: usize = 4_096;
const MAX_REPLAY_EVENTS: usize = 4_096;

/// One event kind captured in a run snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotEvent {
    pub kind: String,
}

impl SnapshotEvent {
    pub fn new(kind: impl Into<String>) -> Self {
        Self { kind: kind.into() }
    }
}

/// Immutable facts and stored events available to a condition evaluation.
///
/// The evaluator borrows this value and never refreshes or mutates it.  Callers can therefore
/// retain the exact snapshot used for an earlier decision and reproduce that decision later.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RunSnapshot {
    pub verify_passed: bool,
    pub verify_exit_code: i64,
    pub iteration: i64,
    pub elapsed: i64,
    pub tokens_used: i64,
    pub events: Vec<SnapshotEvent>,
    pub artifact_exists: bool,
    pub task_status: String,
    pub mail_pending: bool,
}

/// The bounded event shape used by pure historical condition replay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredConditionEvent {
    pub stream_pos: u64,
    pub kind: String,
    pub snapshot: RunSnapshot,
}

impl StoredConditionEvent {
    pub fn new(stream_pos: u64, kind: impl Into<String>, snapshot: RunSnapshot) -> Self {
        Self {
            stream_pos,
            kind: kind.into(),
            snapshot,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ReplayError {
    #[error("historical condition replay exceeds {MAX_REPLAY_EVENTS} events")]
    TooManyEvents,
    #[error("historical events must have strictly increasing stream positions")]
    UnorderedEvents,
}

/// Rebuild the projection visible at `stream_pos` without I/O. The full log.entries integration
/// belongs to the later event/projector slice.
pub fn rebuild_snapshot_as_of(
    events: &[StoredConditionEvent],
    stream_pos: u64,
) -> Result<RunSnapshot, ReplayError> {
    if events.len() > MAX_REPLAY_EVENTS {
        return Err(ReplayError::TooManyEvents);
    }
    let mut previous = None;
    let mut snapshot = RunSnapshot::default();
    for event in events {
        if previous.is_some_and(|position| event.stream_pos <= position) {
            return Err(ReplayError::UnorderedEvents);
        }
        previous = Some(event.stream_pos);
        if event.stream_pos <= stream_pos {
            snapshot = event.snapshot.clone();
        }
    }
    snapshot.events = events
        .iter()
        .filter(|event| event.stream_pos <= stream_pos)
        .map(|event| SnapshotEvent::new(event.kind.clone()))
        .collect();
    Ok(snapshot)
}

/// Evaluate a parsed condition against the historical projection at `stream_pos`.
pub fn evaluate_replayed(
    condition: &Condition,
    events: &[StoredConditionEvent],
    stream_pos: u64,
) -> Result<bool, ReplayError> {
    Ok(condition.evaluate(&rebuild_snapshot_as_of(events, stream_pos)?))
}

/// A parsed and validated condition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Condition {
    expression: Expr,
}

impl Condition {
    /// Parse a condition with `not`, comparisons, `and`, and `or` precedence.
    pub fn parse(source: &str) -> Result<Self, ConditionError> {
        let mut parser = Parser::new(source)?;
        let expression = parser.parse_or(0)?;
        if parser.peek() != &Token::End {
            return Err(ConditionError::Malformed(
                "expected the end of the expression",
            ));
        }
        Ok(Self { expression })
    }

    /// Evaluate this condition against exactly the supplied run snapshot.
    pub fn evaluate(&self, snapshot: &RunSnapshot) -> bool {
        self.expression.evaluate(snapshot)
    }

    /// Short alias for callers treating conditions as predicates.
    pub fn eval(&self, snapshot: &RunSnapshot) -> bool {
        self.evaluate(snapshot)
    }
}

/// Parse one condition expression.
pub fn parse_condition(source: &str) -> Result<Condition, ConditionError> {
    Condition::parse(source)
}

/// Operators supported by the condition language.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Expr {
    Comparison {
        operand: ConditionOperand,
        operator: ComparisonOperator,
        value: Literal,
    },
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Literal {
    Bool(bool),
    Number(i64),
    String(String),
}

impl Expr {
    fn evaluate(&self, snapshot: &RunSnapshot) -> bool {
        match self {
            Self::Comparison {
                operand,
                operator,
                value,
            } => compare(*operand, *operator, value, snapshot),
            Self::Not(expression) => !expression.evaluate(snapshot),
            Self::And(left, right) => left.evaluate(snapshot) && right.evaluate(snapshot),
            Self::Or(left, right) => left.evaluate(snapshot) || right.evaluate(snapshot),
        }
    }
}

fn compare(
    operand: ConditionOperand,
    operator: ComparisonOperator,
    value: &Literal,
    snapshot: &RunSnapshot,
) -> bool {
    match operand {
        ConditionOperand::VerifyPassed => compare_bool(snapshot.verify_passed, operator, value),
        ConditionOperand::VerifyExitCode => {
            compare_number(snapshot.verify_exit_code, operator, value)
        }
        ConditionOperand::Iteration => compare_number(snapshot.iteration, operator, value),
        ConditionOperand::Elapsed => compare_number(snapshot.elapsed, operator, value),
        ConditionOperand::TokensUsed => compare_number(snapshot.tokens_used, operator, value),
        ConditionOperand::ToolErrorCount => compare_number(
            snapshot
                .events
                .iter()
                .take(MAX_SNAPSHOT_EVENTS)
                .filter(|event| event.kind == "tool_error")
                .count() as i64,
            operator,
            value,
        ),
        ConditionOperand::LastEventKind => {
            let last = snapshot
                .events
                .iter()
                .take(MAX_SNAPSHOT_EVENTS)
                .next_back()
                .map(|event| event.kind.as_str());
            compare_string(last.unwrap_or_default(), operator, value)
        }
        ConditionOperand::ArtifactExists => compare_bool(snapshot.artifact_exists, operator, value),
        ConditionOperand::TaskStatus => compare_string(&snapshot.task_status, operator, value),
        ConditionOperand::MailPending => compare_bool(snapshot.mail_pending, operator, value),
    }
}

fn compare_bool(actual: bool, operator: ComparisonOperator, value: &Literal) -> bool {
    let Literal::Bool(expected) = value else {
        return false;
    };
    match operator {
        ComparisonOperator::Equal => actual == *expected,
        ComparisonOperator::NotEqual => actual != *expected,
        _ => false,
    }
}

fn compare_number(actual: i64, operator: ComparisonOperator, value: &Literal) -> bool {
    let Literal::Number(expected) = value else {
        return false;
    };
    match operator {
        ComparisonOperator::Equal => actual == *expected,
        ComparisonOperator::NotEqual => actual != *expected,
        ComparisonOperator::Less => actual < *expected,
        ComparisonOperator::Greater => actual > *expected,
        ComparisonOperator::LessOrEqual => actual <= *expected,
        ComparisonOperator::GreaterOrEqual => actual >= *expected,
    }
}

fn compare_string(actual: &str, operator: ComparisonOperator, value: &Literal) -> bool {
    let Literal::String(expected) = value else {
        return false;
    };
    match operator {
        ComparisonOperator::Equal => actual == expected,
        ComparisonOperator::NotEqual => actual != expected,
        _ => false,
    }
}

/// Errors returned before a condition can be evaluated.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ConditionError {
    #[error("condition is empty")]
    Empty,
    #[error("unknown condition operand `{0}`")]
    UnknownOperand(String),
    #[error("malformed condition: {0}")]
    Malformed(&'static str),
    #[error("condition nesting exceeds {MAX_NESTING} levels")]
    NestingTooDeep,
    #[error("condition exceeds the bounded expression size")]
    ExpressionTooLarge,
    #[error("literal has the wrong type for condition operand `{operand}`")]
    WrongLiteralType { operand: &'static str },
    #[error("invalid number literal `{0}`")]
    InvalidNumber(String),
    #[error("invalid string literal")]
    InvalidString,
    #[error("invalid character `{0}`")]
    InvalidCharacter(char),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Token {
    Word(String),
    String(String),
    Number(i64),
    Bool(bool),
    Operator(ComparisonOperator),
    And,
    Or,
    Not,
    LeftParen,
    RightParen,
    End,
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    nodes: usize,
}

impl Parser {
    fn new(source: &str) -> Result<Self, ConditionError> {
        if source.trim().is_empty() {
            return Err(ConditionError::Empty);
        }
        Ok(Self {
            tokens: lex(source)?,
            position: 0,
            nodes: 0,
        })
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn next(&mut self) -> Token {
        let token = self.tokens[self.position].clone();
        if self.position + 1 < self.tokens.len() {
            self.position += 1;
        }
        token
    }

    fn parse_or(&mut self, depth: usize) -> Result<Expr, ConditionError> {
        self.ensure_depth(depth)?;
        let mut expression = self.parse_and(depth + 1)?;
        while *self.peek() == Token::Or {
            self.next();
            expression = Expr::Or(Box::new(expression), Box::new(self.parse_and(depth + 1)?));
            self.add_node()?;
        }
        Ok(expression)
    }

    fn parse_and(&mut self, depth: usize) -> Result<Expr, ConditionError> {
        self.ensure_depth(depth)?;
        let mut expression = self.parse_not(depth + 1)?;
        while *self.peek() == Token::And {
            self.next();
            expression = Expr::And(Box::new(expression), Box::new(self.parse_not(depth + 1)?));
            self.add_node()?;
        }
        Ok(expression)
    }

    fn parse_not(&mut self, depth: usize) -> Result<Expr, ConditionError> {
        self.ensure_depth(depth)?;
        if *self.peek() == Token::Not {
            self.next();
            let expression = self.parse_not(depth + 1)?;
            self.add_node()?;
            return Ok(Expr::Not(Box::new(expression)));
        }
        self.parse_primary(depth + 1)
    }

    fn parse_primary(&mut self, depth: usize) -> Result<Expr, ConditionError> {
        self.ensure_depth(depth)?;
        if *self.peek() == Token::LeftParen {
            self.next();
            let expression = self.parse_or(depth + 1)?;
            if *self.peek() != Token::RightParen {
                return Err(ConditionError::Malformed("missing closing parenthesis"));
            }
            self.next();
            return Ok(expression);
        }

        let operand = self.parse_operand()?;
        let Token::Operator(operator) = self.next() else {
            return Err(ConditionError::Malformed("expected a comparison operator"));
        };
        let value = self.parse_literal()?;
        validate_literal(operand, &value)?;
        self.add_node()?;
        Ok(Expr::Comparison {
            operand,
            operator,
            value,
        })
    }

    fn parse_operand(&mut self) -> Result<ConditionOperand, ConditionError> {
        let Token::Word(name) = self.next() else {
            return Err(ConditionError::Malformed("expected a condition operand"));
        };

        let operand = match name.as_str() {
            "events.count" => {
                self.expect_word("tool_error", "events.count(tool_error)")?;
                ConditionOperand::ToolErrorCount
            }
            "events.last" => {
                self.expect_word("kind", "events.last(kind)")?;
                ConditionOperand::LastEventKind
            }
            "artifact.exists" => {
                self.expect_word("kind", "artifact.exists(kind)")?;
                ConditionOperand::ArtifactExists
            }
            _ => ConditionOperand::ALL
                .iter()
                .copied()
                .find(|operand| operand.as_str() == name)
                .ok_or(ConditionError::UnknownOperand(name))?,
        };
        Ok(operand)
    }

    fn expect_word(&mut self, expected: &str, operand: &'static str) -> Result<(), ConditionError> {
        if self.next() != Token::LeftParen {
            return Err(ConditionError::UnknownOperand(operand.into()));
        }
        if self.next() != Token::Word(expected.into()) {
            return Err(ConditionError::UnknownOperand(operand.into()));
        }
        if self.next() != Token::RightParen {
            return Err(ConditionError::UnknownOperand(operand.into()));
        }
        Ok(())
    }

    fn parse_literal(&mut self) -> Result<Literal, ConditionError> {
        match self.next() {
            Token::Bool(value) => Ok(Literal::Bool(value)),
            Token::Number(value) => Ok(Literal::Number(value)),
            Token::String(value) => Ok(Literal::String(value)),
            Token::Word(value) => Ok(Literal::String(value)),
            _ => Err(ConditionError::Malformed("expected a literal")),
        }
    }

    fn ensure_depth(&self, depth: usize) -> Result<(), ConditionError> {
        if depth > MAX_NESTING {
            Err(ConditionError::NestingTooDeep)
        } else {
            Ok(())
        }
    }

    fn add_node(&mut self) -> Result<(), ConditionError> {
        self.nodes += 1;
        if self.nodes > MAX_TOKENS / 2 {
            Err(ConditionError::ExpressionTooLarge)
        } else {
            Ok(())
        }
    }
}

fn validate_literal(operand: ConditionOperand, value: &Literal) -> Result<(), ConditionError> {
    let valid = match operand {
        ConditionOperand::VerifyPassed
        | ConditionOperand::ArtifactExists
        | ConditionOperand::MailPending => matches!(value, Literal::Bool(_)),
        ConditionOperand::VerifyExitCode
        | ConditionOperand::Iteration
        | ConditionOperand::Elapsed
        | ConditionOperand::TokensUsed
        | ConditionOperand::ToolErrorCount => matches!(value, Literal::Number(_)),
        ConditionOperand::LastEventKind | ConditionOperand::TaskStatus => {
            matches!(value, Literal::String(_))
        }
    };
    if valid {
        Ok(())
    } else {
        Err(ConditionError::WrongLiteralType {
            operand: operand.as_str(),
        })
    }
}

fn lex(source: &str) -> Result<Vec<Token>, ConditionError> {
    if source.chars().count() > MAX_EXPRESSION_LENGTH {
        return Err(ConditionError::ExpressionTooLarge);
    }
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut position = 0;
    while position < chars.len() {
        if tokens.len() >= MAX_TOKENS {
            return Err(ConditionError::ExpressionTooLarge);
        }
        match chars[position] {
            character if character.is_whitespace() => position += 1,
            '(' => {
                tokens.push(Token::LeftParen);
                position += 1;
            }
            ')' => {
                tokens.push(Token::RightParen);
                position += 1;
            }
            '=' | '!' | '<' | '>' => {
                let first = chars[position];
                let second = chars.get(position + 1).copied();
                let operator = match (first, second) {
                    ('=', Some('=')) => ComparisonOperator::Equal,
                    ('!', Some('=')) => ComparisonOperator::NotEqual,
                    ('<', Some('=')) => ComparisonOperator::LessOrEqual,
                    ('>', Some('=')) => ComparisonOperator::GreaterOrEqual,
                    ('<', _) => ComparisonOperator::Less,
                    ('>', _) => ComparisonOperator::Greater,
                    _ => {
                        return Err(ConditionError::Malformed(
                            "expected a two-character equality operator",
                        ))
                    }
                };
                let width = matches!(second, Some('=')).then_some(2).unwrap_or(1);
                tokens.push(Token::Operator(operator));
                position += width;
            }
            '"' => {
                let start = position;
                position += 1;
                let mut escaped = false;
                let mut closed = false;
                while position < chars.len() {
                    let character = chars[position];
                    position += 1;
                    if escaped {
                        escaped = false;
                    } else if character == '\\' {
                        escaped = true;
                    } else if character == '"' {
                        closed = true;
                        break;
                    }
                }
                if !closed {
                    return Err(ConditionError::InvalidString);
                }
                let literal: String = chars[start..position].iter().collect();
                let value =
                    serde_json::from_str(&literal).map_err(|_| ConditionError::InvalidString)?;
                tokens.push(Token::String(value));
            }
            '-' | '0'..='9' => {
                let start = position;
                position += 1;
                while chars
                    .get(position)
                    .is_some_and(|character| character.is_ascii_digit())
                {
                    position += 1;
                }
                let value: String = chars[start..position].iter().collect();
                let number = value
                    .parse()
                    .map_err(|_| ConditionError::InvalidNumber(value.clone()))?;
                tokens.push(Token::Number(number));
            }
            character if character.is_ascii_alphabetic() || character == '_' => {
                let start = position;
                position += 1;
                while chars.get(position).is_some_and(|character| {
                    character.is_ascii_alphanumeric() || *character == '_' || *character == '.'
                }) {
                    position += 1;
                }
                let word: String = chars[start..position].iter().collect();
                tokens.push(match word.as_str() {
                    "and" => Token::And,
                    "or" => Token::Or,
                    "not" => Token::Not,
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    _ => Token::Word(word),
                });
            }
            character => return Err(ConditionError::InvalidCharacter(character)),
        }
    }
    tokens.push(Token::End);
    Ok(tokens)
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Less => "<",
            Self::Greater => ">",
            Self::LessOrEqual => "<=",
            Self::GreaterOrEqual => ">=",
        })
    }
}

#[cfg(test)]
fn snapshot() -> RunSnapshot {
    RunSnapshot {
        verify_passed: true,
        verify_exit_code: 0,
        iteration: 3,
        elapsed: 42,
        tokens_used: 900,
        events: vec![
            SnapshotEvent::new("tool_error"),
            SnapshotEvent::new("verify"),
        ],
        artifact_exists: true,
        task_status: "ready".into(),
        mail_pending: false,
    }
}

#[cfg(test)]
#[test]
fn parses() {
    for source in [
        "verify.passed == true",
        "verify.exit_code == 0",
        "iteration >= 1",
        "elapsed < 60",
        "tokens.used != 0",
        "events.count(tool_error) == 1",
        "events.last(kind) == verify",
        "artifact.exists(kind) == true",
        "task.status == ready",
        "mail.pending == false",
    ] {
        assert!(Condition::parse(source).is_ok(), "failed to parse {source}");
    }
}

#[test]
fn operators() {
    let facts = snapshot();
    assert!(Condition::parse("not verify.passed == false")
        .unwrap()
        .evaluate(&facts));
    assert!(
        Condition::parse("verify.passed == true and iteration >= 3 or iteration == 0")
            .unwrap()
            .evaluate(&facts)
    );
    assert!(
        !Condition::parse("verify.passed == true and (iteration > 3 or mail.pending == true)")
            .unwrap()
            .evaluate(&facts)
    );
    for operator in ["==", "!=", "<", ">", "<=", ">="] {
        assert!(Condition::parse(&format!("iteration {operator} 3")).is_ok());
    }
}

#[test]
fn evaluates() {
    let facts = snapshot();
    assert!(Condition::parse(
        "verify.passed == true and events.count(tool_error) == 1 and task.status == ready"
    )
    .unwrap()
    .evaluate(&facts));
    assert!(!Condition::parse("mail.pending == true")
        .unwrap()
        .evaluate(&facts));
    assert!(Condition::parse("events.last(kind) == verify")
        .unwrap()
        .evaluate(&facts));
}

#[test]
fn rejects_unknown_operand() {
    let error = Condition::parse("run.status == ready").unwrap_err();
    assert!(matches!(error, ConditionError::UnknownOperand(_)));
    assert!(Condition::parse("events.count(other_error) == 1").is_err());
    assert!(Condition::parse("events.count").is_err());
    assert!(Condition::parse("events.count(").is_err());
    assert!(Condition::parse("iteration = 1").is_err());
    assert!(Condition::parse("iteration >=").is_err());
}

#[test]
fn reproducible() {
    let condition = Condition::parse("events.count(tool_error) == 1 and iteration == 3").unwrap();
    let facts = snapshot();
    let first = condition.evaluate(&facts);
    let second = condition.evaluate(&facts.clone());
    assert_eq!(first, second);
}

#[test]
fn reproducible_historical_replay() {
    let condition = Condition::parse("iteration == 2 and events.count(tool_error) == 1").unwrap();
    let first = RunSnapshot {
        iteration: 1,
        ..RunSnapshot::default()
    };
    let second = RunSnapshot {
        iteration: 2,
        ..RunSnapshot::default()
    };
    let events = vec![
        StoredConditionEvent::new(10, "assistant", first),
        StoredConditionEvent::new(20, "tool_error", second),
    ];
    assert!(!evaluate_replayed(&condition, &events, 10).unwrap());
    assert!(evaluate_replayed(&condition, &events, 20).unwrap());
    assert_eq!(rebuild_snapshot_as_of(&events, 10).unwrap().iteration, 1);
}

#[test]
fn is_total() {
    let deep = "(".repeat(MAX_NESTING + 2) + "iteration == 1" + &")".repeat(MAX_NESTING + 2);
    assert_eq!(Condition::parse(&deep), Err(ConditionError::NestingTooDeep));
    assert!(matches!(
        Condition::parse(&"iteration == 1 ".repeat(700)),
        Err(ConditionError::ExpressionTooLarge)
    ));
    assert!(!Condition::parse("verify.exit_code == 1")
        .unwrap()
        .evaluate(&RunSnapshot::default()));
}
