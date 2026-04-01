use std::{f64, iter::Peekable, str::Chars};

use crate::{calc::Calculator, function::Function};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
  Add, // Binary+
  Sub, // Binary-
  Neg, // Unary-
  Pos, // Unary+
  Mul,
  Div,
  Pow,
  Fac, // Factorial
  Mod, // modulos
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Assoc {
  Left,
  Right,
}

impl Operator {
  // get operator from char. last_token is needed for Unary
  pub fn from(c: char, last_token: Option<&Token>) -> Result<Self, String> {
    // is unary?
    if is_unary(c, last_token) {
      return if c == '-' {
        Ok(Operator::Neg)
      } else {
        Ok(Operator::Pos)
      };
    }

    match c {
      '+' => Ok(Operator::Add),
      '-' => Ok(Operator::Sub),
      '/' => Ok(Operator::Div),
      '*' => Ok(Operator::Mul),
      '^' => Ok(Operator::Pow),
      '!' => Ok(Operator::Fac),
      '%' => Ok(Operator::Mod),
      _ => Err(format!("Invalid Operator: {}", c)),
    }
  }

  pub fn precedence(&self) -> u16 {
    match self {
      Operator::Add | Operator::Sub => 1,
      Operator::Mul | Operator::Div | Operator::Mod => 2,
      Operator::Neg | Operator::Pos => 3,
      Operator::Pow => 4,
      Operator::Fac => 5,
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

  pub fn perform_op(&self, num1: f64, num2: f64) -> Result<f64, String> {
    use Operator as OP;

    let result = match self {
      OP::Add => num1 + num2,
      OP::Sub => num1 - num2,
      OP::Mul => num1 * num2,
      OP::Mod => num1 % num2,

      OP::Div => {
        if num2 == 0f64 {
          return Err(format!(
            "Invalid Expression: division by zero ({}/{})",
            num1, num2
          ));
        }

        num1 / num2
      }

      OP::Pow => {
        // num1 ^ -num2 where num1 is 0 is undefined 
        if num1 == 0f64 && num2 <= 0f64 {
          return Err(format!(
            "Invalid Expression: division by zero ({0}^{1} = 1/({0}^{2}) = 1 / {0})",
            num1, num2, num2.abs()
          ))
        }

        f64::powf(num1, num2)
      }

      _ => {
        return Err(format!(
          "Invalid Token: {:?} Must be handled during parser",
          self
        ));
      }
    };

    if result.is_nan() {
      return Err(format!(
        "Invalid Expression: {} {} {} is NaN",
        num1, self, num2
      ));
    }

    Ok(result)
  }
}

impl std::fmt::Display for Operator {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Operator as OP;
    let name = match self {
      OP::Add => "+",
      OP::Sub => "-",
      OP::Neg => "U-",
      OP::Pos => "U+",
      OP::Mul => "*",
      OP::Div => "/",
      OP::Pow => "^",
      OP::Fac => "!",
      OP::Mod => "%",
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

fn is_unary(c: char, last_token: Option<&Token>) -> bool {
  (c == '-' || c == '+')
    && matches!(
      last_token,
      None | Some(Token::LParen | Token::Operator(_) | Token::Comma)
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
