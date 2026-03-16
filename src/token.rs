use core::f64;
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
  Fac, // Factorial
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Function {
  // 1-arg functions
  Sin,
  Cos,
  Tan,
  Sqrt,
  Abs,
  Ln,
  Exp,
  Floor,
  Ceil,
  Round,
  Recip,
  Cbrt,
  Log,
  // 2-arg functions
  // LogBase,
  // Max,
  // Min,
  // Pow, // it is x ^ y by default but can also be called via pow(x, y)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Constant {
  PI,
  E,   // Euler's number
  INF, // infinity
}

#[derive(Debug, PartialEq)]
pub enum Token {
  Operator(Operator),
  Number(f64),
  Identifier(String), // a word is an identifier before being a function/constant
  Function(Function),
  Constant(Constant),
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
      '!' => Some(Operator::Fac),
      _ => None,
    }
  }

  pub fn precedence(&self) -> u16 {
    match self {
      Operator::Add | Operator::Sub => 1,
      Operator::Mul | Operator::Div => 2,
      Operator::Neg | Operator::Pos => 3,
      Operator::Pow => 4,
      Operator::Fac => 5,
    }
  }

  pub fn get_op_name(&self) -> &'static str {
    match self {
      Operator::Add => "+",
      Operator::Sub => "-",
      Operator::Neg => "U-",
      Operator::Pos => "U+",
      Operator::Mul => "*",
      Operator::Div => "/",
      Operator::Pow => "^",
      Operator::Fac => "!",
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
    match self {
      Operator::Add => Ok(num1 + num2),
      Operator::Sub => Ok(num1 - num2),
      Operator::Mul => Ok(num1 * num2),
      Operator::Div => {
        if num2 == 0f64 {
          Err(format!(
            "Invalid Expression: division by zero ({}/{})",
            num1, num2
          ))
        } else {
          Ok(num1 / num2)
        }
      }
      Operator::Pow => Ok(f64::powf(num1, num2)),

      _ => Err(format!(
        "Invalid Token: {:?} Must be handled during eval",
        self
      )),
    }
  }
}

impl Function {
  pub fn from(name: &str) -> Result<Self, String> {
    match name {
      "sin" => Ok(Function::Sin),
      "cos" => Ok(Function::Cos),
      "tan" => Ok(Function::Tan),
      "sqrt" => Ok(Function::Sqrt),
      "abs" => Ok(Function::Abs),
      "ln" => Ok(Function::Ln),
      "exp" => Ok(Function::Exp),
      "floor" => Ok(Function::Floor),
      "ceil" => Ok(Function::Ceil),
      "round" => Ok(Function::Round),
      "recip" => Ok(Function::Recip),
      "cbrt" => Ok(Function::Cbrt),
      "log" => Ok(Function::Log),
      _ => Err(format!("Invalid Function: {}", name)),
    }
  }

  pub fn get_function_name(&self) -> &'static str {
    match self {
      Function::Sin => "sin",
      Function::Cos => "cos",
      Function::Tan => "tan",
      Function::Sqrt => "sqrt",
      Function::Abs => "abs",
      Function::Ln => "ln",
      Function::Exp => "exp",
      Function::Floor => "floor",
      Function::Ceil => "ceil",
      Function::Round => "round",
      Function::Recip => "recip",
      Function::Cbrt => "cbrt",
      Function::Log => "log",
    }
  }

  pub fn arity(&self) -> usize {
    // all of them only have 1 argument for now
    1
  }

  pub fn call_function(&self, args: &[f64]) -> Result<f64, String> {
    if args.len() != self.arity() {
      return Err(format!(
        "Invalid Arguments: {} takes {} but got {}",
        self.get_function_name(),
        self.arity(),
        args.len()
      ));
    }

    let result = match self {
      Function::Sin => args[0].sin(),
      Function::Cos => args[0].cos(),
      Function::Tan => args[0].tan(),
      Function::Sqrt => args[0].sqrt(),
      Function::Abs => args[0].abs(),
      Function::Ln => args[0].ln(),
      Function::Exp => args[0].exp(),
      Function::Floor => args[0].floor(),
      Function::Ceil => args[0].ceil(),
      Function::Round => args[0].round(),
      Function::Recip => {
        if args[0] == 0f64 {
          return Err(format!(
            "Invalid Expression: division by zero recip({0}) (1/{0})",
            args[0]
          ));
        }

        args[0].recip()
      }
      Function::Cbrt => args[0].cbrt(),
      Function::Log => args[0].log10(),
    };

    if result.is_nan() {
      return Err(format!(
        "Invalid Expression: {}({})",
        self.get_function_name(),
        args[0]
      ));
    }

    Ok(result)
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
    if let Some(op) = Operator::from(c, last_token) {
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

pub fn to_f64(iter: &mut Peekable<Chars>) -> Result<f64, String> {
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
