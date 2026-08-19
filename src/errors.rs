#[derive(Debug, PartialEq, Clone)]
pub struct Span {
    start: usize,
    end: usize,
}

#[derive(Debug)]
pub enum LexerErrorKind {
    InvalidToken(char),
    InvalidNumber(String),
}

#[derive(Debug)]
pub struct LexerError {
    kind: LexerErrorKind,
    span: Span,
    note: Option<String>,
}

impl LexerError {
    pub fn new(kind: LexerErrorKind, span: Span) -> Self {
        Self {
            kind,
            span,
            note: None,
        }
    }

    pub fn with_note(kind: LexerErrorKind, span: Span, note: &str) -> Self {
        Self {
            kind,
            span,
            note: Some(note.to_owned()),
        }
    }
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

impl Diagnostic for LexerError {
    fn span(&self) -> &Span {
        &self.span
    }

    fn message(&self) -> String {
        match &self.kind {
            LexerErrorKind::InvalidToken(c) => format!("invalid token {}", c),
            LexerErrorKind::InvalidNumber(num) => format!("invalid number: '{}'", num),
        }
    }

    fn note(&self) -> Option<String> {
        if self.note.is_some() {
            return self.note.clone();
        }

        match &self.kind {
            LexerErrorKind::InvalidNumber(n) => {
                let exp_count = n.matches(['e', 'E']).count();

                let has_dot_after_exp = n
                    .split(['e', 'E'])
                    .nth(1)
                    .is_some_and(|rest| rest.contains('.'));

                if exp_count > 1 {
                    Some("numbers can only have one exponent".into())
                } else if has_dot_after_exp {
                    Some("exponent cannot contain a decimal point".into())
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    fn prefix(&self) -> &'static str {
        "Lexer Error"
    }
}

trait Diagnostic {
    fn span(&self) -> &Span;
    fn message(&self) -> String;
    fn note(&self) -> Option<String> {
        None
    }
    fn prefix(&self) -> &'static str {
        "Error"
    }
}

#[allow(private_bounds)]
pub fn render_error(src: &str, err: impl Diagnostic + std::fmt::Debug) -> String {
    let span = err.span();
    let prefix = err.prefix();
    let message = err.message();
    let note = err.note();

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
    let out = format!("{prefix}: {message}\n {line}\n {indent}{carets}{note_str}");

    out
}
