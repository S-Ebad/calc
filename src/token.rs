use std::{f64, iter::Peekable, str::Chars};

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
  E, // Euler's number
}

#[derive(Debug, PartialEq)]
pub enum Token {
  Operator(Operator),
  Number(f64),
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
      Operator::Div => Ok(num1 / num2),
      Operator::Pow => Ok(f64::powf(num1, num2)),

      _ => Err(format!(
        "Invalid Operator: {:?} - Must be handled during eval",
        self
      )),
    }
  }
}

impl Function {
  fn from(name: &str) -> Result<Self, String> {
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
      _ => Err(format!("Invalid function: {}", name)),
    }
  }

  pub fn arity(&self) -> usize {
    // all of them only have 1 argument for now
    1
  }

  pub fn call_function(&self, args: &[f64]) -> Result<f64, String> {
    if args.len() != self.arity() {
      return Err(format!(
        "Invalid Arguments: Function {:?} needs {} but given {}",
        self,
        self.arity(),
        args.len()
      ));
    }

    match self {
      Function::Sin => Ok(args[0].sin()),
      Function::Cos => Ok(args[0].cos()),
      Function::Tan => Ok(args[0].tan()),
      Function::Sqrt => Ok(args[0].sqrt()),
      Function::Abs => Ok(args[0].abs()),
      Function::Ln => Ok(args[0].ln()),
      Function::Exp => Ok(args[0].exp()),
      Function::Floor => Ok(args[0].floor()),
      Function::Ceil => Ok(args[0].ceil()),
      Function::Round => Ok(args[0].round()),
      Function::Recip => Ok(args[0].recip()),
      Function::Cbrt => Ok(args[0].cbrt()),
      Function::Log => Ok(args[0].log10()),
    }
  }
}

impl Constant {
  fn from(name: &str) -> Result<Self, String> {
    match name.to_lowercase().as_str() {
      "pi" => Ok(Constant::PI),
      "e" => Ok(Constant::E),
      _ => Err(format!("Invalid Constant: {}", name)),
    }
  }

  pub fn get_number(&self) -> f64 {
    match self {
      Constant::PI => f64::consts::PI,
      Constant::E => f64::consts::E,
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
      return Ok(Token::Number(to_f64(iter)));
    }

    if c.is_alphabetic() {
      let word: String = take_while(iter, |c| c.is_alphabetic());

      return Function::from(&word)
        .map(Token::Function)
        .or_else(|_| Constant::from(&word).map(Token::Constant))
        .map_err(|_| format!("Invalid token: {}", word));
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
