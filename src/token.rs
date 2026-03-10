use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
  Add,
  Sub,
  Mul,
  Div,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
  Operator(Operator),
  Number(f64),
  LParen,
  RParen,
}

impl Operator {
  pub fn from(c: char) -> Option<Self> {
    match c {
      '+' => Some(Operator::Add),
      '-' => Some(Operator::Sub),
      '/' => Some(Operator::Div),
      '*' => Some(Operator::Mul),
      _ => None,
    }
  }
}

impl Token {
  pub fn from(c: char, iter: &mut Peekable<Chars>) -> Result<Self, String> {
    if let Some(op) = Operator::from(c) {
      iter.next();

      return Ok(Token::Operator(op));
    }

    if c.is_digit(10) {
      return Ok(Token::Number(to_f64(iter)));
    }

    if c.is_alphabetic() {
      let word = take_while(iter, |c| c.is_alphabetic());

      return Err(format!("Invalid token: {}", word));
    }

    let result = match c {
      '(' => Ok(Token::LParen),
      ')' => Ok(Token::RParen),

      _ => Err(format!("Invalid token: '{}'", c)),
    };

    iter.next();
    result
  }
}

pub fn take_while<F>(iter: &mut Peekable<Chars>, cond: F) -> String
where
  F: Fn(char) -> bool,
{
  let mut num: String = String::new();

  while let Some(&c) = iter.peek() {
    if !cond(c) {
      break;
    }

    num.push(c);
    iter.next();
  }

  num
}

pub fn to_f64(iter: &mut Peekable<Chars>) -> f64 {
  let num = take_while(iter, |c| c.is_numeric() || c == '.');

  num.parse().unwrap()
}

pub fn tokenize(expr: &str) -> Vec<Token> {
  let mut tokens = Vec::<Token>::new();

  let mut iter = expr.chars().peekable();

  while let Some(&c) = iter.peek() {
    // skip whitespace
    if c.is_whitespace() {
      iter.next();

      continue;
    }

    match Token::from(c, &mut iter) {
      Ok(token) => tokens.push(token),
      Err(err) => eprintln!("Error: {}", err),
    };
  }

  tokens
}
