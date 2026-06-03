use std::collections::HashMap;

use crate::{expr::Expr, user_function::UserFunction};

pub fn eval(
    expr: Expr,
    vars: &HashMap<String, f64>,
    funcs: &HashMap<String, UserFunction>,
) -> Result<f64, String> {
    match expr {
        Expr::Number(num) => Ok(num),
        Expr::Binary { op, lhs, rhs } => {
            let lhs = eval(*lhs, vars, funcs)?;
            let rhs = eval(*rhs, vars, funcs)?;

            op.perform_op(lhs, Some(rhs))
        }
        Expr::Unary { op, expr } => op.perform_op(eval(*expr, vars, funcs)?, None),
        Expr::Postfix { expr, op } => op.perform_op(eval(*expr, vars, funcs)?, None),
        Expr::Call { func, args } => {
            let args = args
                .into_iter()
                .map(|expr| eval(expr, vars, funcs))
                .collect::<Result<Vec<f64>, _>>()?;

            func.call(&args)
        }

        Expr::If {
            condition: condit,
            then,
            else_,
        } => {
            if eval(*condit, vars, funcs)? != 0.0 {
                eval(*then, vars, funcs)
            } else {
                eval(*else_, vars, funcs)
            }
        }

        Expr::UserCall { .. } => {
            todo!()
        }
    }
}
