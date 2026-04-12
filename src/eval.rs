use std::collections::VecDeque;

use crate::{calc::Calculator, operator::Operator, token::Token};

fn factorial(n: f64) -> Result<f64, String> {
    if n < 0.0 || n.fract() != 0.0 {
        return Err(format!("Invalid Argument: factorial undefined for {}", n));
    }

    if n > 170.0 {
        return Err(format!("Invalid Argument: {}! is too large", n));
    }

    Ok((1..=n as u64).map(|x| x as f64).product())
}

impl Calculator {
    pub fn eval(&mut self, mut tokens: VecDeque<Token>) -> Result<f64, String> {
        let mut sum_stk: Vec<f64> = Vec::new();

        while let Some(token) = tokens.pop_front() {
            match token {
                Token::Operator(Operator::Neg) => match sum_stk.pop() {
                    Some(num) => sum_stk.push(-num),
                    None => {
                        return Err("Invalid Operand: negation requires an operand.".to_string());
                    }
                },

                Token::Operator(Operator::Fac) => match sum_stk.pop() {
                    Some(num) => sum_stk.push(factorial(num)?),
                    None => {
                        return Err("Invalid Operand: factorial requires an operand".to_string());
                    }
                },

                Token::Operator(op) => {
                    let (num2, num1) = match (sum_stk.pop(), sum_stk.pop()) {
                        (Some(a), Some(b)) => (a, b),
                        _ => return Err("Invalid Expression: not enough operands".to_string()),
                    };

                    let result = op.perform_op(num1, num2)?;

                    sum_stk.push(result);
                }

                Token::Number(num) => sum_stk.push(num),

                Token::Function(func) => {
                    let arg_count = sum_stk
                        .pop()
                        .ok_or("Invalid Argument: argument count not supplied")?
                        as usize;

                    let mut args: Vec<f64> = Vec::with_capacity(arg_count);

                    if arg_count <= sum_stk.len() {
                        for _ in 0..arg_count {
                            args.insert(0, sum_stk.pop().unwrap());
                        }
                    }

                    sum_stk.push(func.call(&args)?);
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
}
