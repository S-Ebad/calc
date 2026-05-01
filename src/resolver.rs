use std::collections::HashMap;

use crate::{expression::Expression, operator::Operator, user_function::UserFunction};

macro_rules! err_ident {
    ($ident:expr) => {{
        if $ident == "ans" {
            Err("Invalid Identifier: ans not yet defined".to_string())
        } else {
            Err(format!("Invalid Identifier: unknown identifier '{}'", $ident))
        }
    }};
}

pub fn resolver(
    expr: &mut Expression,
    vars: &HashMap<String, f64>,
    funcs: &HashMap<String, UserFunction>,
    depth: u32,
) -> Result<(), String> {
    if depth > 100 {
        return Err("Invalid Expression: Hit recursion limit".to_string());
    }

    match expr {
        Expression::Binary { op: _, lhs, rhs } => {
            resolver(lhs, vars, funcs, depth + 1)?;
            resolver(rhs, vars, funcs, depth + 1)?;
        }
        Expression::Unary { op: _, expr } | Expression::Postfix { expr, op: _ } => {
            resolver(expr, vars, funcs, depth + 1)?;
        }

        Expression::Call { func: _, args } => {
            args.iter_mut()
                .try_for_each(|x| resolver(x, vars, funcs, depth + 1))?;
        }

        Expression::Constant(cons) => {
            *expr = Expression::Number(cons.get_number());
        }

        Expression::UserCall { func, args } => {
            let args = args
                .iter_mut()
                .map(|expr| {
                    resolver(expr, vars, funcs, depth + 1)?;

                    expr.eval()
                })
                .collect::<Result<Vec<f64>, _>>()?;

            let func_def = funcs
                .get(func)
                .ok_or_else(|| format!("Invalid Expression: {} is undefined", func))?;

            let body = func_def.clone().inline(&args)?;

            *expr = *body;
            resolver(expr, vars, funcs, depth + 1)?;
        }

        // check if its a variable
        Expression::Identifier(ident) => {
            // variable
            if let Some(&var) = vars.get(ident) {
                *expr = Expression::Number(var);
            } else {
                return err_ident!(ident);
            }
        }

        // check if its a variable implicit mul
        Expression::Apply { identifier, args } => {
            if let Some(&var) = vars.get(identifier) {
                if args.len() != 1 {
                    return Err(format!(
                        "Invalid Application: Cannot multiply {} by multiple expressions ({})",
                        identifier,
                        args.iter_mut()
                            .map(|a| format!("{}", a))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }

                let mut rhs = args.pop().unwrap();
                resolver(&mut rhs, vars, funcs, depth + 1)?;

                let lhs = Expression::Number(var);

                *expr = Expression::Binary {
                    op: Operator::Mul,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
            } else if let Some(func_def) = funcs.get(identifier) {
                let args = args
                    .iter_mut()
                    .map(|expr| {
                        resolver(expr, vars, funcs, depth + 1)?;

                        expr.eval()
                    })
                    .collect::<Result<Vec<f64>, _>>()?;

                let body = func_def.clone().inline(&args)?;

                *expr = *body;
                resolver(expr, vars, funcs, depth + 1)?;

            } else {
                return Err(format!("Invalid Identifier: unknown function or identifier '{}'", identifier))
            }
        }

        _ => (),
    };

    Ok(())
}
