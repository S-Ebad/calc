use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
  Add, // Binary+
  Sub, // Binary-
  Neg, // Unary-
  Pos, // Unary+
  Mul,
  Div,
  Pow,
}

#[derive(Debug, PartialEq)]
pub enum Token {
  Operator(Operator),
  Number(f64),
  Function(String),
  LParen,
  RParen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Assoc {
  Left,
  Right,
}

impl Operator {
  // get operator from char. last_token is needed for Unary
  pub fn from(c: char, last_token: Option<&Token>) -> Option<Self> {
    // is unary?
    if is_unary(c, last_token) {
      return if c == '-' {
        Some(Operator::Neg)
      } else {
        Some(Operator::Pos)
      };
    }

    match c {
      '+' => Some(Operator::Add),
      '-' => Some(Operator::Sub),
      '/' => Some(Operator::Div),
      '*' => Some(Operator::Mul),
      '^' => Some(Operator::Pow),
      _ => None,
    }
  }

  pub fn precedence(&self) -> u16 {
    match self {
      Operator::Add | Operator::Sub => 1,
      Operator::Mul | Operator::Div => 2,
      Operator::Neg | Operator::Pos => 3,
      Operator::Pow => 4,
    }
  }

  fn associativity(&self) -> Assoc {
    match self {
      Operator::Pow | Operator::Neg | Operator::Pos => Assoc::Right,
      _ => Assoc::Left,
    }
  }

  pub fn is_left_assoc(&self) -> bool {
    matches!(self.associativity(), Assoc::Left)
  }
}

impl Token {
  pub fn from(
    c: char,
    iter: &mut Peekable<Chars>,
    last_token: Option<&Token>,
  ) -> Result<Self, String> {
    if let Some(op) = Operator::from(c, last_token) {
      iter.next();

      return Ok(Token::Operator(op));
    }

    if c.is_digit(10) {
      return Ok(Token::Number(to_f64(iter)));
    }

    if c.is_alphabetic() {
      let word: String = take_while(iter, |c| c.is_alphabetic());

      return Ok(Token::Function(word));
    }

    let result = match c {
      '(' => Ok(Token::LParen),
      ')' => Ok(Token::RParen),

      _ => Err(format!("Invalid token: '{}' ({})", c, c as i8)),
    };

    iter.next();
    result
  }
}

fn is_unary(c: char, last_token: Option<&Token>) -> bool {
  (c == '-' || c == '+')
    && matches!(
      last_token,
      None | Some(Token::LParen) | Some(Token::Operator(_))
    )
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

    match Token::from(c, &mut iter, tokens.last()) {
      Ok(token) => tokens.push(token),
      Err(err) => eprintln!("Error: {}", err),
    };
  }

  tokens
}
