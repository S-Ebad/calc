use std::{fmt, iter::Peekable, str::Chars};

use crate::{constant::Constant, err_fmt, function::Function, operator::Operator};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
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

#[derive(Debug)]
pub struct Lexer {
    tokens: Vec<Token>,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name: &str = match self {
            Token::Operator(operator) => &operator.to_string(),
            Token::Number(n) => &n.to_string(),
            Token::Identifier(name) => name,
            Token::Function(func) => &func.to_string(),
            Token::Comma => ",",
            Token::LParen => "(",
            Token::RParen => ")",
            Token::QuestionMark => "?",
            Token::Colon => ":",
            Token::Dot => ".",
            Token::Constant(constant) => &constant.to_string(),
        };

        write!(f, "{}", name)
    }
}

impl Lexer {
    pub fn new(src: &str) -> Result<Self, String> {
        let mut tokens = tokenize(src)?;
        tokens.reverse();

        Ok(Lexer { tokens })
    }

    pub fn peek(&mut self) -> Option<&Token> {
        self.tokens.last()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.pop()
    }
}

impl Token {
    pub fn from(c: char, iter: &mut Peekable<Chars>) -> Result<Self, String> {
        if let Some(op) = Operator::from(c, iter) {
            iter.next();

            return Ok(Token::Operator(op));
        }

        if c.is_ascii_digit() {
            return Ok(Token::Number(to_f64(iter)?));
        }

        if c.is_alphabetic() {
            let mut word: String = take_while(iter, |c| c.is_alphabetic() || c == '_');

            // atan2
            if word == "atan" && iter.peek() == Some(&'2') {
                word.push(iter.next().unwrap());
            }

            if let Some(constant) = Constant::from(&word) {
                return Ok(Token::Constant(constant));
            }

            return Ok(Token::Identifier(word));
        }

        let result = match c {
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            ',' => Ok(Token::Comma),
            '?' => Ok(Token::QuestionMark),
            ':' => Ok(Token::Colon),
            '.' => Ok(Token::Dot),

            _ => err_fmt!("Lexer Error: invalid token '{}'", c),
        };

        iter.next();
        result
    }

    pub fn left_bp(&self) -> u8 {
        match self {
            Token::Operator(op) => op.bp().0,
            Token::LParen | Token::Identifier(_) | Token::Number(_) | Token::Constant(_) => {
                Operator::ImplicitMul.bp().0 // Uses 11
            }

            // bind as tight as postfix
            Token::Dot => Operator::Fac.bp().0,

            //binds between Equal and everything else
            Token::QuestionMark => 1,

            _ => 0,
        }
    }
}

// take_while but doesn't consume an extra element
fn take_while<F>(iter: &mut Peekable<Chars>, cond: F) -> String
where
    F: Fn(char) -> bool,
{
    let mut s: String = String::new();

    while let Some(&c) = iter.peek() {
        if !cond(c) {
            break;
        }

        s.push(c);
        iter.next();
    }

    s
}

fn to_f64(iter: &mut Peekable<Chars>) -> Result<f64, String> {
    let mut num = take_while(iter, |c| c.is_ascii_digit() || c == '.');

    let mut lookahead = iter.clone();
    let is_exponent = if matches!(lookahead.peek(), Some('e' | 'E')) {
        lookahead.next();

        if matches!(lookahead.peek(), Some('-' | '+')) {
            lookahead.next();
        }

        matches!(lookahead.peek(), Some(c) if c.is_ascii_digit())
    } else {
        false
    };

    if is_exponent {
        num.push(iter.next().unwrap()); // 'e'

        if matches!(iter.peek(), Some('-') | Some('+')) {
            num.push(iter.next().unwrap());
        }

        num.push_str(&take_while(iter, |c| c.is_ascii_digit()));

        // things like 9e9e9 or 9e9.2 which are invalid
        if matches!(iter.peek(), Some('e' | '.' | 'E')) {
            return Err(format!(
                "Lexer Error: invalid number '{num}{}'",
                iter.peek().unwrap()
            ));
        }
    }

    num.parse::<f64>()
        .map_err(|_| format!("Lexer Error: invalid number '{num}'"))
}

fn tokenize(expr: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::<Token>::new();

    let mut iter = expr.chars().peekable();

    while let Some(&c) = iter.peek() {
        // skip whitespace
        if c.is_whitespace() {
            iter.next();

            continue;
        }

        tokens.push(Token::from(c, &mut iter)?);
    }

    Ok(tokens)
}
