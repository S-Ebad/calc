use crate::rawexpr::join;
use crate::{expr::Expr, operator::Operator, rawexpr::RawExpr, user_function::UserFunction};
use std::collections::HashMap;

macro_rules! err_fmt {
    ($fmt:expr, $($arg:expr),* $(,)?) => {
        Err(format!($fmt, $($arg),*))
    };
}

macro_rules! err_ident {
    ($ident:expr) => {{
        if $ident == "ans" {
            Err("Resolver Error: ans not yet defined".to_string())
        } else {
            err_fmt!("Resolver Error: Unknown identifier '{}'", $ident)
        }
    }};
}

pub fn resolve(
    expr: RawExpr,
    vars: &HashMap<String, f64>,
    funcs: &HashMap<String, UserFunction>,
) -> Result<Expr, String> {

    let expr = match expr {
        RawExpr::Number(n) => Expr::Number(n),
        RawExpr::Constant(constant) => Expr::Number(constant.get_number()),
        RawExpr::Binary { op, lhs, rhs } => {
            let lhs = resolve(*lhs, vars, funcs)?;
            let rhs = resolve(*rhs, vars, funcs)?;

            Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }
        RawExpr::Unary { op, expr } => {
            let expr = resolve(*expr, vars, funcs)?;

            Expr::Unary {
                op,
                expr: Box::new(expr),
            }
        }
        RawExpr::Postfix { op, expr } => {
            let expr = resolve(*expr, vars, funcs)?;

            Expr::Postfix {
                op,
                expr: Box::new(expr),
            }
        }
        RawExpr::Apply { name, mut args } => {
            if let Some(var) = vars.get(&name) {
                if args.len() != 1 {
                    return err_fmt!(
                        "Resolver Error: Cannot multiply {} by multiple expressions ({})",
                        name,
                        join(args.iter(), ", "),
                    );
                }

                let arg = args.pop().unwrap();
                let rhs = resolve(arg, vars, funcs)?;

                let lhs = Expr::Number(*var);

                Expr::Binary {
                    op: Operator::ImplicitMul,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            } else if funcs.contains_key(&name) {
                let resolved_args = args
                    .into_iter()
                    .map(|raw_expr| resolve(raw_expr, vars, funcs))
                    .collect::<Result<Vec<Expr>, _>>()?;

                Expr::UserCall {
                    func: name,
                    args: resolved_args,
                }
            } else {
                return err_fmt!("Resolver Error: Unknown identifier '{}'", name);
            }
        }

        RawExpr::If {
            condition,
            then,
            else_,
        } => {
            let condition = resolve(*condition, vars, funcs)?;
            let then = resolve(*then, vars, funcs)?;
            let else_ = resolve(*else_, vars, funcs)?;

            Expr::If {
                condition: Box::new(condition),
                then: Box::new(then),
                else_: Box::new(else_),
            }
        }
        RawExpr::Identifier(ident) => {
            if let Some(var) = vars.get(&ident) {
                Expr::Number(*var)
            } else {
                return err_ident!(ident);
            }
        }
    };

    Ok(expr)
}

/*
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

    // Expression::UserCall { func, args } => {
    //     let args = args
    //         .iter_mut()
    //         .map(|expr| {
    //             resolver(expr, vars, funcs, depth + 1)?;
    //
    //             expr.eval()
    //         })
    //         .collect::<Result<Vec<f64>, _>>()?;
    //
    //     let func_def = funcs
    //         .get(func)
    //         .ok_or_else(|| format!())?;
    //
    //     let body = func_def.clone().inline(&args)?;
    //
    //     *expr = *body;
    //     resolver(expr, vars, funcs, depth + 1)?;
    // }

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
            *expr = Expression::UserCall {
                func: func_def.name.clone(),
                args: args.clone(),
            };

        } else {
            return Err(format!(
                "Invalid Identifier: unknown function or identifier '{}'",
                identifier
            ));
        }
    }

    Expression::If {
        condition,
        then,
        else_,
    } => {
        resolver(condition, vars, funcs, depth + 1)?;
        resolver(then, vars, funcs, depth + 1)?;
        resolver(else_, vars, funcs, depth + 1)?;
    }

    _ => (),
};
*/

// Ok(())
