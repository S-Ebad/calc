use crate::{lexer::Token, raw_expr::RawExpr};

#[derive(Debug)]
pub enum CalcError {
    Lexer(LexerError),
    Parser(ParseError),
    String(String), // Keeping this temporarily until ResolverError & EvalError are implemented
}

impl Diagnostic for CalcError {
    fn span(&self) -> &Option<Span> {
        match self {
            CalcError::Lexer(l) => l.span(),
            CalcError::Parser(p) => p.span(),
            CalcError::String(_) => &None,
        }
    }

    fn message(&self) -> String {
        match self {
            CalcError::Lexer(l) => l.message(),
            CalcError::Parser(p) => p.message(),
            CalcError::String(s) => s.to_owned(),
        }
    }

    fn note(&self) -> &Option<String> {
        match self {
            CalcError::Lexer(l) => l.note(),
            CalcError::Parser(p) => p.note(),
            CalcError::String(_) => &None,
        }
    }

    fn prefix(&self) -> &'static str {
        match self {
            CalcError::Lexer(l) => l.prefix(),
            CalcError::Parser(p) => p.prefix(),
            CalcError::String(_) => "",
        }
    }
}

impl From<LexerError> for CalcError {
    fn from(value: LexerError) -> Self {
        Self::Lexer(value)
    }
}

impl From<ParseError> for CalcError {
    fn from(value: ParseError) -> Self {
        Self::Parser(value)
    }
}

impl From<String> for CalcError {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, b: Self) -> Self {
        let start = self.start.min(b.start);
        let end = self.end.max(b.end);

        Self { start, end }
    }
}

#[derive(Debug)]
pub enum LexerErrorKind {
    InvalidToken(char),
    InvalidNumber(String),
}

trait Diagnostic {
    fn span(&self) -> &Option<Span>;
    fn message(&self) -> String;
    fn note(&self) -> &Option<String> {
        &None
    }
    fn prefix(&self) -> &'static str {
        "Error"
    }
}

#[derive(Debug)]
pub struct LexerError {
    kind: LexerErrorKind,
    span: Option<Span>,
    note: Option<String>,
}

impl LexerError {
    pub fn new(kind: LexerErrorKind, span: Option<Span>) -> Self {
        Self {
            kind,
            span,
            note: None,
        }
    }

    pub fn with_note(kind: LexerErrorKind, span: Option<Span>, note: &str) -> Self {
        Self {
            kind,
            span,
            note: Some(note.to_owned()),
        }
    }
}

impl Diagnostic for LexerError {
    fn span(&self) -> &Option<Span> {
        &self.span
    }

    fn message(&self) -> String {
        match &self.kind {
            LexerErrorKind::InvalidToken(c) => format!("invalid token {}", c),
            LexerErrorKind::InvalidNumber(num) => format!("invalid number: '{}'", num),
        }
    }

    fn note(&self) -> &Option<String> {
        &self.note
    }

    fn prefix(&self) -> &'static str {
        "Lexer Error"
    }
}

#[derive(Debug)]
pub enum ParseErrorKind {
    EmptyExpression,
    ExtraClosingParenthesis,
    MissingClosingParenthesis,
    UnexpectedEndOfInput,
    MissingOperator,
    InvalidParameter(RawExpr),
    ExpectedMethodName(Option<Token>),
    ExpectedColonAfterQuestionMark,
    TrailingComma,
    ExpectedClosingParenthesis(Token),
    InvalidAssignmentTarget(String),
    CannotStartExpression(Token),
    UnexpectedToken(Token),
    FunctionUsedAsValue(String),
}

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
    span: Option<Span>,
    note: Option<String>,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, span: Option<Span>) -> Self {
        Self {
            kind,
            span,
            note: None,
        }
    }

    pub fn with_note(kind: ParseErrorKind, span: Option<Span>, note: String) -> Self {
        Self {
            kind,
            span,
            note: Some(note),
        }
    }
}

impl Diagnostic for ParseError {
    fn span(&self) -> &Option<Span> {
        &self.span
    }

    fn message(&self) -> String {
        match &self.kind {
            ParseErrorKind::EmptyExpression => "no expression to parse".to_string(),
            ParseErrorKind::ExtraClosingParenthesis => {
                "unexpected closing parenthesis ')'".to_string()
            }

            ParseErrorKind::UnexpectedToken(token) => format!("unexpected token: {}", token.kind),
            ParseErrorKind::FunctionUsedAsValue(name) => {
                format!("'{}' is a function, not a value", name)
            }

            ParseErrorKind::MissingClosingParenthesis => "unclosed parenthesis".to_string(),
            ParseErrorKind::UnexpectedEndOfInput => "unexpected end of input".to_string(),
            ParseErrorKind::CannotStartExpression(token) => {
                format!("'{}' cannot start an expression", token.kind)
            }

            ParseErrorKind::MissingOperator => "missing operator between expressions".to_string(),

            ParseErrorKind::ExpectedMethodName(token) => match token {
                Some(token) => format!("expected method name after '.', got '{}'", token.kind),
                None => "expected method name after '.', got nothing".to_string(),
            },

            ParseErrorKind::ExpectedColonAfterQuestionMark => "expected ':' after '?'".to_string(),
            ParseErrorKind::TrailingComma => "trailing comma before ')'".to_string(),
            ParseErrorKind::InvalidAssignmentTarget(name) => {
                format!("cannot assign to '{}'", name)
            }
            ParseErrorKind::InvalidParameter(raw_expr) => format!(
                "function parameter must be an identifier, got '{}'",
                raw_expr.kind
            ),
            ParseErrorKind::ExpectedClosingParenthesis(token) => {
                format!("expected closing parenthesis, got '{}'", token.kind)
            }
        }
    }

    fn note(&self) -> &Option<String> {
        &self.note
    }

    fn prefix(&self) -> &'static str {
        "Parse Error"
    }
}

#[allow(private_bounds)]
pub fn render_error(src: &str, err: impl Diagnostic) -> String {
    let prefix = err.prefix();
    let message = err.message();
    let note = err.note().as_deref();

    let out;
    if let Some(span) = err.span() {
        let line_start = src[..span.start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = src[span.end..]
            .find('\n')
            .map(|i| span.end + i)
            .unwrap_or(src.len());

        let line = &src[line_start..line_end];

        let col_start = span.start - line_start;
        let col_end = span.end - line_start;

        let indent = " ".repeat(col_start);
        let carets = "^".repeat((col_end - col_start).max(1));

        let note_str = note.map(|n| format!(" {n}")).unwrap_or_default();

        out = format!("{prefix}: {message}\n {line}\n {indent}{carets}{note_str}");
    } else {
        out = format!("{prefix}: {message}");
    }

    out
}
