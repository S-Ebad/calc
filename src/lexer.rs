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
        if let Some((op, should_consume)) = Operator::from(c, iter.peek().cloned()) {
            if should_consume {
                iter.next();
            }

            return Ok(Token::Operator(op));
        }

        if c.is_ascii_digit() {
            return Ok(Token::Number(to_f64(c, iter)?));
        }

        if c.is_alphabetic() {
            let mut word: String = accumulate(c.to_string(), iter, |c| c.is_alphabetic() || c == '_');

            // atan2
            if word == "atan" && iter.peek() == Some(&'2') {
                word.push(iter.next().unwrap());
            }

            if let Some(constant) = Constant::from(&word) {
                return Ok(Token::Constant(constant));
            }

            return Ok(Token::Identifier(word));
        }

        match c {
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            ',' => Ok(Token::Comma),
            '?' => Ok(Token::QuestionMark),
            ':' => Ok(Token::Colon),

            _ => err_fmt!("Lexer Error: invalid token '{}'", c)?,
        }
    }

    pub fn left_bp(&self) -> u8 {
        match self {
            Token::Operator(op) => op.bp().0,
            Token::LParen | Token::Identifier(_) | Token::Number(_) | Token::Constant(_) => {
                Operator::ImplicitMul.bp().0 // Uses 11
            }

            //binds between Equal and everything else
            Token::QuestionMark => 1,

            _ => 0,
        }
    }
}

/// like take_while, but seeded with a string
fn accumulate<F>(mut seed: String, iter: &mut Peekable<Chars>, cond: F) -> String
where
    F: Fn(char) -> bool,
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

fn to_f64(c: char, iter: &mut Peekable<Chars>) -> Result<f64, String> {
    let mut num = accumulate(c.to_string(), iter, |c| c.is_numeric() || c == '.');

    // differentiate between 9 * e (euler's number) and 9e9
    let mut mul_euler = false;
    if iter.peek() == Some(&'e') {
        iter.next();

        if iter
            .peek()
            .map(|c| c.is_numeric() || *c == '-')
            .unwrap_or(false)
        {
            num.push('e');

            if iter.peek() == Some(&'-') {
                num.push(iter.next().unwrap());
            }

            // accept more e & . to invalidate expressions like 9e9e9 or 9e9.9
            num.push_str(&accumulate(String::new(), iter, |c| {
                c.is_numeric() || c == 'e' || c == '.'
            }))
        } else {
            mul_euler = true;
        }
    }

    let result = num
        .parse::<f64>()
        .map_err(|_| format!("Lexer Error: invalid number '{}'", num));

    if mul_euler {
        Ok(result? * std::f64::consts::E)
    } else {
        result
    }
}

fn tokenize(expr: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::<Token>::new();

    let mut iter = expr.chars().peekable();

    while let Some(c) = iter.next() {
        if c.is_whitespace() {
            continue;
        }

        tokens.push(Token::from(c, &mut iter)?);
    }

    Ok(tokens)
}
