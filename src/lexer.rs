use std::{fmt, ops::Deref};

use crate::{
    constant::Constant, err_fmt, errors::Span, function::Function, operator::Operator,
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
    kind: TokenKind,
    span: Span,
}

#[derive(Debug)]
pub struct Lexer {
    tokens: Vec<Token>,
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
    pub fn new(src: &str) -> Result<Self, String> {
        let mut tokens = tokenize(src)?;
        tokens.reverse();

        Ok(Lexer { tokens })
    }

    pub fn peek(&mut self) -> Option<&TokenKind> {
        self.tokens.last().map(|token| &token.kind)
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

impl Iterator for Lexer {
    type Item = TokenKind;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.pop().map(|token| token.kind)
    }
}

impl TokenKind {
    pub fn from<I>(c: char, iter: &mut PosChars<I>) -> Result<Self, String>
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
            return Ok(TokenKind::Number(to_f64(c, iter)?));
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

            _ => err_fmt!("Lexer Error: invalid token '{}'", c)?,
        }
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

fn to_f64<I>(c: char, iter: &mut PosChars<I>) -> Result<f64, String>
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

            if let Some(bad @ ('e' | 'E')) = iter.peek() {
                return Err(format!("Lexer Erorr: invalid number '{num}{bad}'"));
            }

            if iter.peek() == Some(&'.') && peek_at(iter, 1).is_some_and(|c| c.is_ascii_digit()) {
                let dot = iter.next().unwrap();
                return Err(format!("Lexer Error: invalid number: '{num}{dot}'"));
            }
        }
    }

    num.parse::<f64>()
        .map_err(|_| format!("Lexer Error: invalid number '{num}'"))
}

fn tokenize(expr: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::<Token>::new();
    let mut iter = crate::poschars::PosChars::new(expr.chars().peekable());

    while let Some(c) = iter.next() {
        if c.is_whitespace() {
            continue;
        }

        let start = iter.pos();
        let token = TokenKind::from(c, &mut iter)?;
        let end = iter.pos();

        tokens.push(Token { kind: token, span: Span::new(start, end) });
    }

    Ok(tokens)
}
