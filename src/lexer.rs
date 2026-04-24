use std::{iter::Peekable, str::Chars};

use crate::constant::Constant;
use crate::{function::Function, operator::Operator};

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
}

pub struct Lexer {
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(src: &str) -> Result<Self, String> {
        let mut tokens = tokenize(src)?;
        tokens.reverse();

        Ok(Lexer { tokens })
    }

    // consume next
    pub fn next(&mut self) -> Option<Token> {
        self.tokens.pop()
    }

    pub fn peek(&mut self) -> Option<&Token> {
        self.tokens.last()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

impl Token {
    pub fn from(
        c: char,
        iter: &mut Peekable<Chars>,
    ) -> Result<Self, String> {
        if let Ok(op) = Operator::from(c) {
            iter.next();

            return Ok(Token::Operator(op));
        }

        if c.is_ascii_digit() {
            return Ok(Token::Number(to_f64(iter)?));
        }

        if c.is_alphabetic() {
            let mut word: String = take_while(iter, |c| c.is_alphabetic());

            // atan2
            if word == "atan" && iter.peek() == Some(&'2') {
                word.push(iter.next().unwrap());
            }

            return Ok(Token::Identifier(word));
        }

        let result = match c {
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            ',' => Ok(Token::Comma),

            _ => Err(format!("Invalid Token: '{}'", c)),
        };

        iter.next();
        result
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
    let mut num = take_while(iter, |c| c.is_numeric() || c == '.');

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
            num.push_str(&take_while(iter, |c| {
                c.is_numeric() || c == 'e' || c == '.'
            }))
        } else {
            mul_euler = true;
        }
    }

    let result = num
        .parse::<f64>()
        .map_err(|_| format!("Invalid Number: '{}'", num));

    if mul_euler {
        Ok(result? * std::f64::consts::E)
    } else {
        result
    }
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

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Constant::PI => "PI",
            Constant::E => "e",
            Constant::Inf => "Inf",
        };

        write!(f, "{}", name)
    }
}
