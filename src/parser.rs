use std::collections::VecDeque;

use crate::token::Token;

pub fn shunting_yard(token: &Token, stk_op: &mut Vec<Token>, que_out: &mut VecDeque<Token>) {
  // TODO: Add Unary Minus/Plus

  match token {
    Token::Operator(op) => {
      while let Some(top) = stk_op.last() {
        match top {
          Token::Operator(op2) => {
            if op2.precedence() < op.precedence() {
              break;
            }
          }

          _ => break,
        };

        let last = stk_op.pop().expect("Stack underflow");
        que_out.push_back(last);
      }

      stk_op.push(*token);
    }
    Token::Number(_) => {
      que_out.push_back(*token);
    }
    Token::LParen => {
      stk_op.push(*token);
    }
    Token::RParen => {
      while let Some(top) = stk_op.last() {
        if matches!(top, Token::LParen) {
          break;
        }

        let last = stk_op.pop().expect("Stack underflow");
        que_out.push_back(last);
      }

      if stk_op.pop().is_none() {
        eprintln!("Error: mismatched parentheses");
      }
    }
  }
}

pub fn parse(tokens: &Vec<Token>) {
  println!("tokens: {}", tokens.len());

  let mut stk_op: Vec<Token> = Vec::new();
  let mut que_out: VecDeque<Token> = VecDeque::new();

  for token in tokens.iter() {
    shunting_yard(&token, &mut stk_op, &mut que_out);
  }

  while let Some(last) = stk_op.pop() {
    que_out.push_back(last);
  }

  for token in que_out.iter() {
    println!("{:?}", token);
  }
}
