use std::collections::HashMap;

use crate::constant::Constant;
use crate::expression::{Expression, IntoFunction};
use crate::function::Function;
use crate::operator::Operator;

macro_rules! err_ident {
    ($ident:expr) => {{
        if $ident == "ans" {
            Err("Invalid Identifier: ans not yet defined".to_string())
        } else {
            Err(format!("Invalid Identifier: {}", $ident))
        }
    }};
}

fn resolve_ident(ident: &str, vars: &HashMap<String, f64>) -> Result<Expression, String> {
    if let Ok(cons) = Constant::from(ident) {
        Ok(Expression::Number(cons.get_number()))
    } else if let Some(var) = vars.get(ident) {
        Ok(Expression::Number(*var))
    } else {
        return err_ident!(ident);
    }
}

pub fn resolver(
    expr: &mut Expression,
    vars: &HashMap<String, f64>,
    funcs: &HashMap<String, IntoFunction>,
) -> Result<(), String> {
    match expr {
        Expression::Binary {
            op: op @ (Operator::ImplicitMul | Operator::Sub),
            lhs,
            rhs,
        } => {
            // sin x?
            let mut rhs_owned = rhs.clone();
            let mut lhs_owned = lhs.clone();

            let res_op = if op == &mut Operator::ImplicitMul {
                Operator::Mul
            } else {
                Operator::Sub
            };

            *expr = match (lhs.as_ref(), rhs.as_ref()) {
                (Expression::Identifier(ident), _) => {
                    resolver(&mut rhs_owned, vars, funcs)?;

                    if let Ok(func) = Function::from(ident) {
                        let arg = if op == &Operator::Sub {
                            Expression::Unary {
                                op: Operator::Neg,
                                expr: rhs_owned,
                            }
                        } else {
                            *rhs_owned
                        };

                        Expression::Call {
                            func,
                            args: vec![arg],
                        }
                    } else {
                        Expression::Binary {
                            op: res_op,
                            lhs: Box::new(resolve_ident(ident, vars)?),
                            rhs: rhs_owned,
                        }
                    }
                }

                (_, Expression::Identifier(ident)) => {
                    if let Ok(_) = Function::from(ident) {
                        return Err(format!("Cannot call {} on rhs of implicit mul", ident));
                    }

                    resolver(&mut lhs_owned, vars, funcs)?;

                    Expression::Binary {
                        op: res_op,
                        lhs: lhs_owned,
                        rhs: Box::new(resolve_ident(ident, vars)?),
                    }
                }

                _ => {
                    resolver(&mut lhs_owned, vars, funcs)?;
                    resolver(&mut rhs_owned, vars, funcs)?;

                    Expression::Binary {
                        op: res_op,
                        lhs: lhs_owned,
                        rhs: rhs_owned,
                    }
                }
            };

            Ok(())
        }
        Expression::Binary { op: _, lhs, rhs } => {
            resolver(lhs, vars, funcs)?;
            resolver(rhs, vars, funcs)
        }

        Expression::Unary { op: _, expr } | Expression::Postfix { expr, op: _ } => {
            resolver(expr, vars, funcs)
        }

        Expression::Apply {
            identifier: ident,
            args,
        } => {
            if let Ok(func) = Function::from(ident) {
                let mut resolved_args = args.clone();
                for arg in resolved_args.iter_mut() {
                    resolver(arg, vars, funcs)?;
                }

                *expr = Expression::Call {
                    func,
                    args: resolved_args,
                };
            } else if let Ok(cons) = Constant::from(ident) {
                if args.len() != 1 {
                    return Err(format!(
                        "Invalid Application: Cannot multiply {} by multiple expressions ({})",
                        ident,
                        args.into_iter()
                            .map(|a| format!("{}", a))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }

                let mut rhs_expr = args[0].clone();
                resolver(&mut rhs_expr, vars, funcs)?;

                let num = Expression::Number(cons.get_number());
                *expr = Expression::Binary {
                    op: Operator::Mul,
                    lhs: Box::new(num),
                    rhs: Box::new(rhs_expr),
                };
            } else if let Some(var) = vars.get(ident) {
                if args.len() != 1 {
                    return Err(format!(
                        "Invalid Application: Cannot multiply {} by multiple expressions ({})",
                        ident,
                        args.into_iter()
                            .map(|a| format!("{}", a))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }

                let mut rhs_expr = args[0].clone();
                resolver(&mut rhs_expr, vars, funcs)?;

                let num = Expression::Number(*var);
                *expr = Expression::Binary {
                    op: Operator::Mul,
                    lhs: Box::new(num),
                    rhs: Box::new(rhs_expr),
                };
            } else {
                return err_ident!(ident);
            }

            Ok(())
        }

        Expression::Identifier(ident) => {
            if let Ok(_) = Function::from(ident) {
                Err(format!(
                    "Invalid Expression: Cannot use function '{}' as a value",
                    ident
                ))
            } else if let Ok(cons) = Constant::from(ident) {
                *expr = Expression::Number(cons.get_number());
                Ok(())
            } else if let Some(var) = vars.get(ident) {
                *expr = Expression::Number(*var);
                Ok(())
            } else {
                err_ident!(ident)
            }
        }

        _ => Ok(()),
    }
}
