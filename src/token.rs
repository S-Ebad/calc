use std::{f64, iter::Peekable, str::Chars};

use crate::{calc::Calculator, function::Function, operator::Operator};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Constant {
    PI,
    E,   // Euler's number
    INF, // infinity
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Operator(Operator),
    Number(f64),
    Identifier(String), // a word is an identifier before being a function/constant/variable
    Function(Function),
    Constant(Constant),
    Assign, // equal
    Comma,
    LParen,
    RParen,
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Constant::PI => "PI",
            Constant::E => "e",
            Constant::INF => "Inf",
        };

        write!(f, "{}", name)
    }
}

impl Constant {
    pub fn from(name: &str) -> Result<Self, String> {
        match name.to_lowercase().as_str() {
            "pi" => Ok(Constant::PI),
            "e" => Ok(Constant::E),
            "inf" => Ok(Constant::INF),
            _ => Err(format!("Invalid Constant: {}", name)),
        }
    }

    pub fn get_number(&self) -> f64 {
        match self {
            Constant::PI => f64::consts::PI,
            Constant::E => f64::consts::E,
            Constant::INF => f64::INFINITY,
        }
    }
}

impl Token {
    pub fn from(
        c: char,
        iter: &mut Peekable<Chars>,
        last_token: Option<&Token>,
    ) -> Result<Self, String> {
        if let Ok(op) = Operator::from(c, last_token) {
            iter.next();

            return Ok(Token::Operator(op));
        }

        if c.is_digit(10) {
            return Ok(Token::Number(to_f64(iter)?));
        }

        if c.is_alphabetic() {
            let word: String = take_while(iter, |c| c.is_alphabetic());

            return Ok(Token::Identifier(word));
        }

        let result = match c {
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            ',' => Ok(Token::Comma),
            '=' => Ok(Token::Assign),

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
        Ok(result? * f64::consts::E)
    } else {
        result
    }
}

impl Calculator {
    pub fn tokenize(expr: &str) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::<Token>::new();

        let mut iter = expr.chars().peekable();

        while let Some(&c) = iter.peek() {
            // skip whitespace
            if c.is_whitespace() {
                iter.next();

                continue;
            }

            tokens.push(Token::from(c, &mut iter, tokens.last())?);
        }

        Ok(tokens)
    }
}
