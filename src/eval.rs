use std::collections::VecDeque;

use crate::token::{Operator, Token};

fn factorial(n: f64) -> Result<f64, String> {
  if n < 0.0 || n.fract() != 0.0 {
    return Err(format!("Invalid Argument: factorial undefined for {}", n));
  }

  if n > 170.0 {
    return Err(format!("Invalid Argument: {}! is too large", n));
  }

  Ok((1..=n as u64).map(|x| x as f64).product())
}

pub fn eval(mut tokens: VecDeque<Token>) -> Result<f64, String> {
  let mut sum_stk: Vec<f64> = Vec::new();

  while let Some(token) = tokens.pop_front() {
    match token {
      Token::Operator(Operator::Neg) => match sum_stk.pop() {
        Some(num) => sum_stk.push(-num),
        None => return Err("Invalid Operand: negation requires an operand.".to_string()),
      },

      Token::Operator(Operator::Fac) => match sum_stk.pop() {
        Some(num) => sum_stk.push(factorial(num)?),
        None => return Err("Invalid Operand: factorial requires an operand".to_string()),
      },

      Token::Operator(op) => {
        let (num2, num1) = match (sum_stk.pop(), sum_stk.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => return Err("Invalid Expression: not enough operands".to_string()),
        };

        let result = op.perform_op(num1, num2)?;
        if result.is_nan() {
          return Err(format!("Invalid Expression: {} {} {} is NaN", num1, op.get_op_name(), num2));
        }

        sum_stk.push(result);
      }

      Token::Number(num) => sum_stk.push(num),
      Token::Function(func) => {
        let arg = match sum_stk.pop() {
          Some(n) => n,
          None => return Err(format!("Invalid Argument: {} requires an argument", func.get_function_name())),
        };

        sum_stk.push(func.call_function(&[arg])?);
      }

      _ => {
        return Err(format!(
          "Invalid Token: {:?} Must be handled during parser",
          token
        ));
      }
    }
  }

  if sum_stk.len() != 1 {
    return Err("Invalid Expression: missing operator".to_string());
  }

  Ok(sum_stk.pop().unwrap())
}
