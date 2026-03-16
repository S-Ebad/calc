use std::collections::VecDeque;

use crate::token::{Constant, Function, Operator, Token};

pub fn shunting_yard(
  token: Token,
  op_stk: &mut Vec<Token>,
  out_que: &mut VecDeque<Token>,
) -> Result<(), String> {
  match token {
    Token::Number(_) | Token::Operator(Operator::Fac) => {
      out_que.push_back(token);
    }
    // unwrap constants
    Token::Constant(c) => out_que.push_back(Token::Number(c.get_number())),

    Token::Operator(Operator::Neg) => {
      op_stk.push(token);
    }

    // does nothing. No point carrying it through eval
    Token::Operator(Operator::Pos) => (),

    Token::Operator(op1) => {
      while let Some(Token::Operator(op2)) = op_stk.last() {
        if (op2.precedence() > op1.precedence())
          || (op2.precedence() == op1.precedence() && op1.is_left_assoc())
        {
          let last = op_stk
            .pop()
            .ok_or(format!("Invalid state: operator stack underflow"))?;

          out_que.push_back(last);
        } else {
          break;
        }
      }

      op_stk.push(token);
    }
    /* TODO: Token::Comma (for multi arg functions) i.e. (rand(1,2)) */
    Token::Function(_) | Token::LParen => {
      op_stk.push(token);
    }
    Token::RParen => {
      while let Some(top) = op_stk.last() {
        if matches!(top, Token::LParen) {
          break;
        }

        let last = op_stk
          .pop()
          .ok_or(format!("Invalid state: operator stack underflow"))?;

        out_que.push_back(last);
      }

      // MUST pop parentheses
      if op_stk.pop().is_none() {
        return Err(format!("Invalid Expression: mismatched parentheses"));
      }

      // check if the parentheses was function
      if matches!(op_stk.last(), Some(Token::Function(_))) {
        out_que.push_back(op_stk.pop().unwrap());
      }
    }

    Token::Identifier(_) => (),
  }

  Ok(())
}

fn preprocessor(tokens: Vec<Token>, ans: Option<f64>) -> Result<Vec<Token>, String> {
  let mut result: Vec<Token> = Vec::with_capacity(tokens.len());
  let mut iter = tokens.into_iter().peekable();

  while let Some(curr) = iter.next() {
    let next = iter.peek();

    match curr {
      // Insert implicit * between number/ closing paren and Identifier/opening paren
      // e.g. 2pi -> 2 * pi, 2(3+4) -> 2 * (3+4), etc
      Token::Number(_) | Token::RParen
        if matches!(next, Some(Token::Identifier(_) | Token::LParen)) =>
      {
        result.push(curr);
        result.push(Token::Operator(Operator::Mul));
      }

      Token::Identifier(name) if name == "ans" => match ans {
        Some(val) => result.push(Token::Number(val)),
        None => return Err("Invalid Identifier: ans is not yet defined".to_string()),
      },

      // Resolve identifier to function or constant
      Token::Identifier(name) => {
        // expand previous answer

        if let Ok(f) = Function::from(&name) {
          result.push(Token::Function(f));

          // Implicit function call: sin30 -> sin(30), sin pi -> sin(pi)
          // Only wraps a single token. sin(30+1) requires explicit parens
          if matches!(next, Some(Token::Number(_) | Token::Identifier(_))) {
            result.push(Token::LParen);

            let arg = iter.next().unwrap();
            match arg {
              Token::Identifier(name2) => {
                let constant = Constant::from(&name2).map_err(|_| {
                  format!(
                    "Invalid Argument: {} is not a valid argument for {}",
                    name2, name
                  )
                })?;

                result.push(Token::Constant(constant));
              }
              _ => result.push(arg),
            };

            result.push(Token::RParen);
          }
        } else if let Ok(c) = Constant::from(&name) {
          result.push(Token::Constant(c));

          // Insert implicit * after constant when followed by opening paren or number
          // e.g. pi(x) -> pi * (x), pi5 -> pi * 5
          if matches!(next, Some(Token::LParen | Token::Number(_))) {
            result.push(Token::Operator(Operator::Mul));
          }
        } else {
          return Err(format!("Invalid Identifier: {}", name));
        }
      }
      _ => result.push(curr),
    }
  }

  Ok(result)
}

pub fn parse(tokens: Vec<Token>, ans: Option<f64>) -> Result<VecDeque<Token>, String> {
  if tokens.is_empty() {
    return Err("Invalid Expression: no expression to parse".to_string());
  }

  let tokens = preprocessor(tokens, ans)?;

  let mut op_stk: Vec<Token> = Vec::new();
  let mut out_que: VecDeque<Token> = VecDeque::new();

  for token in tokens.into_iter() {
    shunting_yard(token, &mut op_stk, &mut out_que)?;
  }

  while let Some(last) = op_stk.pop() {
    if matches!(last, Token::RParen | Token::LParen) {
      return Err(format!("Invalid Expression: unclosed parentheses"));
    }

    if let Token::Function(func) = last {
      return Err(format!(
        "Invalid Expression: no arguments for {}",
        func.get_function_name()
      ));
    }

    out_que.push_back(last);
  }

  Ok(out_que)
}
