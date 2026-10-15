use crate::{function::Function, lexer::Token, operator::Operator, raw_expr::RawExpr};

#[derive(Debug)]
pub enum CalcError {
    Lexer(Box<LexerError>),
    Parser(Box<ParseError>),
    Resolver(Box<ResolverError>),
    Eval(Box<EvalError>),
}

impl CalcError {
    // every variant is a Diagnostic, so reporting is pure delegation
    fn inner(&self) -> &dyn Diagnostic {
        match self {
            CalcError::Lexer(e) => e.as_ref(),
            CalcError::Parser(e) => e.as_ref(),
            CalcError::Resolver(e) => e.as_ref(),
            CalcError::Eval(e) => e.as_ref(),
        }
    }
}

impl Diagnostic for CalcError {
    fn span(&self) -> &Option<Span> {
        self.inner().span()
    }

    fn message(&self) -> String {
        self.inner().message()
    }

    fn note(&self) -> &Option<String> {
        self.inner().note()
    }

    fn prefix(&self) -> &'static str {
        self.inner().prefix()
    }
}

impl From<LexerError> for CalcError {
    fn from(value: LexerError) -> Self {
        Self::Lexer(Box::new(value))
    }
}

impl From<ParseError> for CalcError {
    fn from(value: ParseError) -> Self {
        Self::Parser(Box::new(value))
    }
}

impl From<ResolverError> for CalcError {
    fn from(value: ResolverError) -> Self {
        Self::Resolver(Box::new(value))
    }
}

impl From<EvalError> for CalcError {
    fn from(value: EvalError) -> Self {
        Self::Eval(Box::new(value))
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

impl LexerErrorKind {
    pub fn message(&self) -> String {
        match self {
            LexerErrorKind::InvalidToken(c) => format!("invalid token {}", c),
            LexerErrorKind::InvalidNumber(num) => format!("invalid number: '{}'", num),
        }
    }
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

    pub fn kind(&self) -> &LexerErrorKind {
        &self.kind
    }
}

impl Diagnostic for LexerError {
    fn span(&self) -> &Option<Span> {
        &self.span
    }

    fn message(&self) -> String {
        self.kind.message()
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

impl ParseErrorKind {
    pub fn message(&self) -> String {
        match self {
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

    pub fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }
}

impl Diagnostic for ParseError {
    fn span(&self) -> &Option<Span> {
        &self.span
    }

    fn message(&self) -> String {
        self.kind.message()
    }

    fn note(&self) -> &Option<String> {
        &self.note
    }

    fn prefix(&self) -> &'static str {
        "Parse Error"
    }
}

#[derive(Debug)]
pub enum ResolverErrorKind {
    InvalidMultiplication {
        target: String,
        args: Vec<RawExpr>,
    },

    UnknownFunction(String),
    UnknownIdentifier(String),
    UndefinedAns, // when 'ans' is used as the first expression (no previous answer)
    ArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },
}

impl ResolverErrorKind {
    pub fn message(&self) -> String {
        match self {
            ResolverErrorKind::InvalidMultiplication { target, args } => {
                format!(
                    "cannot multiply {} by multiple expressions ({})",
                    target,
                    join_args(args)
                )
            }
            ResolverErrorKind::UnknownFunction(name) => format!("unknown function '{}'", name),
            ResolverErrorKind::ArityMismatch {
                name,
                expected,
                got,
            } => format!(
                "function {} takes {} argument(s) but got {}",
                name, expected, got
            ),
            ResolverErrorKind::UnknownIdentifier(name) => format!("unknown identifier '{}'", name),
            ResolverErrorKind::UndefinedAns => "ans not yet defined".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ResolverError {
    kind: ResolverErrorKind,
    span: Option<Span>,
    note: Option<String>,
}

impl ResolverError {
    pub fn new(kind: ResolverErrorKind, span: Option<Span>) -> Self {
        Self {
            kind,
            span,
            note: None,
        }
    }

    pub fn with_note(kind: ResolverErrorKind, span: Option<Span>, note: String) -> Self {
        Self {
            kind,
            span,
            note: Some(note),
        }
    }

    pub fn kind(&self) -> &ResolverErrorKind {
        &self.kind
    }
}

impl Diagnostic for ResolverError {
    fn span(&self) -> &Option<Span> {
        &self.span
    }

    fn message(&self) -> String {
        self.kind.message()
    }

    fn note(&self) -> &Option<String> {
        &self.note
    }

    fn prefix(&self) -> &'static str {
        "Resolver Error"
    }
}

fn join_args(args: &[RawExpr]) -> String {
    args.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Debug)]
pub enum EvalErrorKind {
    // resolving a user function body is a genuine eval-time step under dynamic
    // scoping, so a resolution failure is one way evaluation can fail.
    // boxed: ResolverErrorKind's widest variant carries a Vec<RawExpr>, and
    // EvalError travels up a recursive eval stack (see 04911b4)
    Resolution(Box<ResolverErrorKind>),
    RecursionLimit,

    DivisionByZero { lhs: f64, rhs: f64 },

    // 0 ^ -n, from both the '^' operator and pow()
    ZeroToNegativePower { base: f64, exp: f64 },

    NotPrefixOrPostfix(Operator),
    AssignmentNotAtTopLevel,

    FactorialUndefined(f64),
    FactorialTooLarge(f64),

    BuiltinArity { func: Function, got: usize },

    ClampRange { min: f64, max: f64 },

    TanAsymptote(f64),
    RecipZero,
    SqrtZeroRoot(f64),

    // the operation is defined but produced NaN
    UndefinedBinary { lhs: f64, op: Operator, rhs: f64 },

    UndefinedCall { func: Function, args: Vec<f64> },
}

impl EvalErrorKind {
    pub fn message(&self) -> String {
        match self {
            EvalErrorKind::Resolution(kind) => kind.message(),
            EvalErrorKind::RecursionLimit => "recursion limit has been reached".to_string(),

            EvalErrorKind::DivisionByZero { lhs, rhs } => {
                format!("{}/{} is undefined (division by zero)", lhs, rhs)
            }

            EvalErrorKind::ZeroToNegativePower { base, exp } => format!(
                "{0}^{1} is undefined (division by zero: 1/{0}^{2})",
                base,
                exp,
                exp.abs()
            ),

            EvalErrorKind::NotPrefixOrPostfix(op) => {
                format!("{} is not a postfix/prefix operator", op)
            }

            EvalErrorKind::AssignmentNotAtTopLevel => {
                "'=' can only be used at the top level for assignment".to_string()
            }

            EvalErrorKind::FactorialUndefined(n) => {
                format!("factorial is undefined for {}", n)
            }

            EvalErrorKind::FactorialTooLarge(n) => {
                format!("{}! is too large (max is 170!)", n)
            }

            EvalErrorKind::BuiltinArity { func, got } => format!(
                "function {} takes {} argument(s) but got {}",
                func,
                func.arity_str(),
                got
            ),

            EvalErrorKind::ClampRange { min, max } => {
                format!("clamp range min ({}) must be less than max ({})", min, max)
            }

            EvalErrorKind::TanAsymptote(n) => format!(
                "function tan({}) is undefined at asymptote (pi / 2 + n * pi)",
                n
            ),

            EvalErrorKind::RecipZero => "recip(0) is undefined (division by zero)".to_string(),

            EvalErrorKind::SqrtZeroRoot(x) => format!(
                "sqrt({0}, 0) is undefined (division by zero: {0} ^ (1/0))",
                x
            ),

            EvalErrorKind::UndefinedBinary { lhs, op, rhs } => {
                format!("{} {} {} is undefined", lhs, op, rhs)
            }

            EvalErrorKind::UndefinedCall { func, args } => format!(
                "{}({}) is undefined",
                func,
                args.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Debug)]
pub struct EvalError {
    kind: EvalErrorKind,
    span: Option<Span>,
    note: Option<String>,
}

impl EvalError {
    pub fn new(kind: EvalErrorKind, span: Option<Span>) -> Self {
        Self {
            kind,
            span,
            note: None,
        }
    }

    pub fn with_note(kind: EvalErrorKind, span: Option<Span>, note: String) -> Self {
        Self {
            kind,
            span,
            note: Some(note),
        }
    }

    pub fn kind(&self) -> &EvalErrorKind {
        &self.kind
    }
}

impl Diagnostic for EvalError {
    fn span(&self) -> &Option<Span> {
        &self.span
    }

    fn message(&self) -> String {
        self.kind.message()
    }

    fn note(&self) -> &Option<String> {
        &self.note
    }

    fn prefix(&self) -> &'static str {
        match &self.kind {
            // the prefix names what went wrong, not when it was noticed
            EvalErrorKind::Resolution(_) => "Resolver Error",
            _ => "Eval Error",
        }
    }
}

impl From<ResolverError> for EvalError {
    fn from(value: ResolverError) -> Self {
        let ResolverError { kind, span, note } = value;

        Self {
            kind: EvalErrorKind::Resolution(Box::new(kind)),
            span,
            note,
        }
    }
}

#[allow(private_bounds)]
pub fn render_error(src: &str, err: impl Diagnostic) -> String {
    let prefix = err.prefix();
    let message = err.message();
    let note = err.note().as_deref();

    // a span is only meaningful against the source it was taken from; refuse to
    // slice with one that doesn't fit rather than panicking on a bad index
    let fits = |span: &Span| {
        span.start <= span.end
            && span.end <= src.len()
            && src.is_char_boundary(span.start)
            && src.is_char_boundary(span.end)
    };

    let Some(span) = err.span().filter(fits) else {
        return format!("{prefix}: {message}");
    };

    let line_start = src[..span.start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = src[span.end..]
        .find('\n')
        .map(|i| span.end + i)
        .unwrap_or(src.len());

    let line = &src[line_start..line_end];

    // char counts, not byte offsets, so a multi-byte char earlier in the line
    // doesn't shift the carets
    let col_start = src[line_start..span.start].chars().count();
    let col_end = src[line_start..span.end].chars().count();

    let indent = " ".repeat(col_start);
    let carets = "^".repeat((col_end - col_start).max(1));

    let note_str = note.map(|n| format!(" {n}")).unwrap_or_default();

    format!("{prefix}: {message}\n {line}\n {indent}{carets}{note_str}")
}
