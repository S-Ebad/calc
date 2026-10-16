use std::{borrow::Cow, fmt, ops::Deref};

use crate::{
    constant::Constant,
    errors::{LexerError, LexerErrorKind, Span},
    function::Function,
    operator::Operator,
    poschars::PosChars,
};

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Operator(Operator),
    Number(f64),
    Identifier(String), // a word is an identifier before being a function/constant/variable
    Function(Function),
    Constant(Constant),
    Comma,
    LParen,
    RParen,

    QuestionMark,
    Colon,
    Dot,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub struct Lexer {
    tokens: Vec<Token>,
    src_len: usize,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name: &str = match self {
            TokenKind::Operator(operator) => &operator.to_string(),
            TokenKind::Number(n) => &n.to_string(),
            TokenKind::Identifier(name) => name,
            TokenKind::Function(func) => &func.to_string(),
            TokenKind::Comma => ",",
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::QuestionMark => "?",
            TokenKind::Colon => ":",
            TokenKind::Dot => ".",
            TokenKind::Constant(constant) => &constant.to_string(),
        };

        write!(f, "{}", name)
    }
}

impl std::ops::DerefMut for Token {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}

impl std::ops::Deref for Token {
    type Target = TokenKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl Token {
    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

impl Lexer {
    pub fn new(src: &str) -> Result<Self, LexerError> {
        let mut tokens = tokenize(src)?;
        tokens.reverse();

        Ok(Self {
            tokens,
            src_len: src.len(),
        })
    }

    pub fn peek(&self) -> Option<&TokenKind> {
        self.tokens.last().map(|token| &token.kind)
    }

    pub fn peek_token(&self) -> Option<&Token> {
        self.tokens.last()
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.tokens.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn end_pos(&self) -> usize {
        self.src_len
    }

    /// The parameter names, if the line is a definition: `name(a, b, ...) = ...`
    ///
    /// The body comes after the parameters it is read in terms of, so the
    /// parser has to know them before it gets there. Whether a name is a call
    /// or a value decides the shape of the tree (`f - 2` against `f(-2)`), and
    /// that cannot be repaired once the body has been built the wrong way.
    ///
    /// Anything other than plain names between the parentheses is not a
    /// definition this can vouch for, and is left to parse as it otherwise
    /// would, errors included.
    pub(crate) fn definition_params(&self) -> Option<Vec<String>> {
        // stored back to front, so popping is cheap
        let mut tokens = self.tokens.iter().rev().map(|token| &token.kind);

        let Some(TokenKind::Identifier(_)) = tokens.next() else {
            return None;
        };

        let Some(TokenKind::LParen) = tokens.next() else {
            return None;
        };

        let mut params = Vec::new();

        loop {
            match tokens.next()? {
                TokenKind::Identifier(name) => params.push(name.clone()),
                TokenKind::RParen if params.is_empty() => break, // f() = ...
                _ => return None,
            }

            match tokens.next()? {
                TokenKind::Comma => continue,
                TokenKind::RParen => break,
                _ => return None,
            }
        }

        matches!(tokens.next(), Some(TokenKind::Operator(Operator::Equal))).then_some(params)
    }
}

impl Iterator for Lexer {
    type Item = TokenKind;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.pop().map(|token| token.kind)
    }
}

impl TokenKind {
    pub fn from<I>(c: char, start: usize, iter: &mut PosChars<I>) -> Result<Self, LexerError>
    where
        I: Iterator<Item = char> + Clone,
    {
        if let Some((op, should_consume)) = Operator::from(c, iter.peek().cloned()) {
            if should_consume {
                iter.next();
            }

            return Ok(TokenKind::Operator(op));
        }

        if c.is_ascii_digit() {
            return Ok(TokenKind::Number(to_f64(c, start, iter)?));
        }

        if c.is_alphabetic() {
            let mut word: String =
                accumulate(c.to_string(), iter, |c| c.is_alphabetic() || c == '_');

            // atan2
            if word == "atan" && iter.peek() == Some(&'2') {
                word.push(iter.next().unwrap());
            }

            if let Some(constant) = Constant::from(&word) {
                return Ok(TokenKind::Constant(constant));
            }

            return Ok(TokenKind::Identifier(word));
        }

        match c {
            '(' => Ok(TokenKind::LParen),
            ')' => Ok(TokenKind::RParen),
            ',' => Ok(TokenKind::Comma),
            '?' => Ok(TokenKind::QuestionMark),
            ':' => Ok(TokenKind::Colon),
            '.' => Ok(TokenKind::Dot),

            _ => Err(LexerError::new(
                LexerErrorKind::InvalidToken(c),
                Some(Span::new(start, start)),
            )),
        }
    }

    /// Whether this token could begin an operand.
    ///
    /// An eager call takes the next operand as its argument, so a name only
    /// reads as a call when something an argument could start with follows it.
    pub fn can_start_operand(&self) -> bool {
        matches!(
            self,
            TokenKind::Number(_)
                | TokenKind::Identifier(_)
                | TokenKind::Constant(_)
                | TokenKind::Function(_)
                | TokenKind::LParen
                | TokenKind::Operator(Operator::Sub | Operator::Add)
        )
    }

    pub fn left_bp(&self) -> u8 {
        match self {
            TokenKind::Operator(op) => op.bp().0,
            TokenKind::LParen
            | TokenKind::Identifier(_)
            | TokenKind::Number(_)
            | TokenKind::Constant(_) => {
                Operator::ImplicitMul.bp().0 // Uses 11
            }

            // bind as tight as postfix
            TokenKind::Dot => Operator::Fac.bp().0,

            //binds between Equal and everything else
            TokenKind::QuestionMark => 1,

            _ => 0,
        }
    }
}

/// like take_while, but seeded with a string
fn accumulate<F, I>(mut seed: String, iter: &mut PosChars<I>, cond: F) -> String
where
    F: Fn(char) -> bool,
    I: Iterator<Item = char>,
{
    while let Some(&c) = iter.peek() {
        if !cond(c) {
            break;
        }

        seed.push(c);
        iter.next();
    }

    seed
}

fn peek_at<I>(iter: &PosChars<I>, offset: usize) -> Option<char>
where
    I: Iterator<Item = char> + Clone,
{
    let mut lookahead = iter.deref().clone();
    for _ in 0..offset {
        lookahead.next();
    }

    lookahead.next()
}

fn to_f64<I>(c: char, start: usize, iter: &mut PosChars<I>) -> Result<f64, LexerError>
where
    I: Iterator<Item = char> + Clone,
{
    let mut num = accumulate(c.to_string(), iter, |c| c.is_ascii_digit());

    if iter.peek() == Some(&'.') && peek_at(iter, 1).is_some_and(|c| c.is_ascii_digit()) {
        num.push(iter.next().unwrap()); // '.'
        num.push_str(&accumulate(String::new(), iter, |c| c.is_ascii_digit()));
    }

    if matches!(iter.peek(), Some('e' | 'E')) {
        let sign_offset = if matches!(peek_at(iter, 1), Some('+' | '-')) {
            2
        } else {
            1
        };

        if peek_at(iter, sign_offset).is_some_and(|c| c.is_ascii_digit()) {
            num.push(iter.next().unwrap()); // e/E

            if matches!(iter.peek(), Some('+' | '-')) {
                num.push(iter.next().unwrap());
            }

            num.push_str(&accumulate(String::new(), iter, |c| c.is_ascii_digit()));

            if let Some('e' | 'E') = iter.peek() {
                let exp_offset = iter.pos();

                // consume the entire "number" so we can give an accurate error
                num.push_str(&accumulate(String::new(), iter, |c| {
                    c.is_ascii_digit() || c == 'E' || c == 'e' || c == '+' || c == '-' || c == '.'
                }));

                let end = iter.pos();
                let span = Span::new(exp_offset, end);
                let kind = LexerErrorKind::InvalidNumber(num);

                return Err(LexerError::with_note(
                    kind,
                    Some(span),
                    "numbers can only have one exponent",
                ));
            }

            // catches decimal points AFTER e
            if iter.peek() == Some(&'.') && peek_at(iter, 1).is_some_and(|c| c.is_ascii_digit()) {
                let dot_offset = iter.pos();

                // consume the entire "number" so we can give an accurate error
                num.push_str(&accumulate(String::new(), iter, |c| {
                    c.is_ascii_digit() || c == 'E' || c == 'e' || c == '+' || c == '-' || c == '.'
                }));

                let end = iter.pos();
                let span = Span::new(dot_offset, end);
                let kind = LexerErrorKind::InvalidNumber(num);

                return Err(LexerError::with_note(
                    kind,
                    Some(span),
                    "exponent cannot contain a decimal point",
                ));
            }
        }
    }

    num.parse::<f64>().map_err(|_| {
        let end = iter.pos();
        let span = Span::new(start, end);

        let kind = LexerErrorKind::InvalidNumber(num);

        LexerError::new(kind, Some(span))
    })
}

fn tokenize(expr: &str) -> Result<Vec<Token>, LexerError> {
    let mut tokens = Vec::<Token>::new();
    let mut iter = crate::poschars::PosChars::new(expr.chars().peekable());

    while let Some(c) = iter.next() {
        if c.is_whitespace() {
            continue;
        }

        if c == COMMENT {
            // the rest of the line is the comment
            for c in iter.by_ref() {
                if c == '\n' {
                    break;
                }
            }

            continue;
        }

        // positions are in bytes, and the char just taken may be several
        let start = iter.pos() - c.len_utf8();
        let token = TokenKind::from(c, start, &mut iter)?;
        let end = iter.pos();

        tokens.push(Token {
            kind: token,
            span: Span::new(start, end),
        });
    }

    Ok(tokens)
}

/// Starts a comment, which runs to the end of its line.
const COMMENT: char = '#';

/// Separates one statement from the next.
const SEPARATOR: char = ';';

/// One statement of a longer input.
pub struct Piece<'a> {
    /// where the statement starts in the input, in bytes
    pub offset: usize,
    pub text: &'a str,

    /// whether anything is left once whitespace and comments are set aside
    pub has_code: bool,
}

/// Split an input into its statements.
///
/// This runs before anything is lexed, so it has to know where comments are
/// as well: a separator inside one is part of the comment.
pub fn statements(src: &str) -> impl Iterator<Item = Piece<'_>> {
    let mut chars = src.char_indices();
    let mut start = 0;
    let mut done = false;

    std::iter::from_fn(move || {
        if done {
            return None;
        }

        let mut has_code = false;
        let mut in_comment = false;

        for (i, c) in chars.by_ref() {
            match c {
                '\n' if in_comment => in_comment = false,
                _ if in_comment => (),

                COMMENT => in_comment = true,

                SEPARATOR => {
                    let piece = Piece {
                        offset: start,
                        text: &src[start..i],
                        has_code,
                    };

                    start = i + SEPARATOR.len_utf8();
                    return Some(piece);
                }

                c if c.is_whitespace() => (),
                _ => has_code = true,
            }
        }

        done = true;

        Some(Piece {
            offset: start,
            text: &src[start..],
            has_code,
        })
    })
}

/// The input with its comments taken out, for the few checks that read the
/// text before it is lexed.
pub fn strip_comments(src: &str) -> Cow<'_, str> {
    if !src.contains(COMMENT) {
        return Cow::Borrowed(src);
    }

    let code = src
        .lines()
        .map(|line| line.split(COMMENT).next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    Cow::Owned(code)
}
