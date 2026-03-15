use std::collections::VecDeque;

use crate::token::{Operator, Token};

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
    Token::Constant(c) => {
      out_que.push_back(Token::Number(c.get_number()))
    }

    Token::Operator(Operator::Neg) => {
      op_stk.push(token);
    }

    // does nothing. No point carrying it through eval
    Token::Operator(Operator::Pos) => (),

    Token::Operator(op1) => {
      while let Some(Token::Operator(op2)) = op_stk.last() {
        if matches!(op2, Operator::Neg) {
          break;
        }

        if (op2.precedence() > op1.precedence())
          || (op2.precedence() == op1.precedence() && op1.is_left_assoc())
        {
          let last = op_stk.pop().expect("Stack underflow");

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

        let last = op_stk.pop().expect("Stack underflow");
        out_que.push_back(last);
      }

      // MUST pop parentheses
      if op_stk.pop().is_none() {
        return Err(format!("Mismatched parentheses"));
      }

      // check if the parentheses was function
      if matches!(op_stk.last(), Some(Token::Function(_))) {
        out_que.push_back(op_stk.pop().unwrap());
      }
    }
  }

  Ok(())
}

pub fn parse(tokens: Vec<Token>) -> Result<VecDeque<Token>, String> {
  let mut op_stk: Vec<Token> = Vec::new();
  let mut out_que: VecDeque<Token> = VecDeque::new();

  for token in tokens.into_iter() {
    shunting_yard(token, &mut op_stk, &mut out_que)?;
  }

  while let Some(last) = op_stk.pop() {
    if matches!(last, Token::RParen | Token::LParen) {
      return Err(format!("Unclosed Parentheses"));
    }

    if let Token::Function(name) = last {
      return Err(format!("No function arguments for {:?}", name));
    }

    out_que.push_back(last);
  }

  Ok(out_que)
}
