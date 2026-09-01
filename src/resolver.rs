use crate::{
    expr::Expr, function::Function, operator::Operator, raw_expr::{RawExpr, RawExprKind}, user_function::UserFunction,
};
use std::{borrow::Borrow, collections::HashMap, hash::Hash};

#[macro_export]
macro_rules! err_fmt {
    ($fmt:literal, $($arg:expr),* $(,)?) => {
        Err(format!($fmt, $($arg),*))
    };

    ($fmt:expr) => {
        Err($fmt.to_string())
    };
}

macro_rules! err_ident {
    ($ident:expr) => {{
        if $ident == "ans" {
            Err("Resolver Error: ans not yet defined".to_string())
        } else {
            err_fmt!("Resolver Error: unknown identifier '{}'", $ident)
        }
    }};
}

impl RawExpr {
    pub fn resolve<K>(
        self,
        vars: &HashMap<K, f64>,
        funcs: &HashMap<String, UserFunction>,
    ) -> Result<Expr, String>
    where
        K: Borrow<str> + Hash + Eq,
    {
        let expr = match self.kind {
            RawExprKind::Number(n) => Expr::Number(n),
            RawExprKind::Binary { op, lhs, rhs } => {
                let lhs = lhs.resolve(vars, funcs)?;
                let rhs = rhs.resolve(vars, funcs)?;

                Expr::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }
            RawExprKind::Unary { op, expr } => {
                let expr = expr.resolve(vars, funcs)?;

                Expr::Unary {
                    op,
                    expr: Box::new(expr),
                }
            }
            RawExprKind::Postfix { op, expr } => {
                let expr = expr.resolve(vars, funcs)?;

                Expr::Postfix {
                    op,
                    expr: Box::new(expr),
                }
            }
            RawExprKind::Apply { name, mut args } => {
                if let Some(var) = vars.get(&name).cloned() {
                    if args.len() != 1 {
                        return err_fmt!(
                            "Resolver Error: cannot multiply {} by multiple expressions ({})",
                            name,
                            args.into_iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }

                    let arg = args.pop().unwrap();
                    let rhs = arg.resolve(vars, funcs)?;

                    let lhs = Expr::Number(var);

                    Expr::Binary {
                        op: Operator::ImplicitMul,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }
                } else if funcs.contains_key(&name) {
                    let resolved_args = args
                        .into_iter()
                        .map(|raw_expr| raw_expr.resolve(vars, funcs))
                        .collect::<Result<Vec<Expr>, _>>()?;

                    Expr::UserCall {
                        name,
                        args: resolved_args,
                    }
                } else if let Some(func) = Function::from(&name) {
                    let resolved_args = args
                        .into_iter()
                        .map(|raw_expr| raw_expr.resolve(vars, funcs))
                        .collect::<Result<Vec<_>, _>>()?;

                    Expr::Call {
                        func,
                        args: resolved_args,
                    }
                } else {
                    return err_fmt!("Resolver Error: unknown function '{}'", name);
                }
            }

            RawExprKind::If {
                condition,
                then,
                else_,
            } => {
                let condition = condition.resolve(vars, funcs)?;
                let then = then.resolve(vars, funcs)?;
                let else_ = else_.resolve(vars, funcs)?;

                Expr::If {
                    condition: Box::new(condition),
                    then: Box::new(then),
                    else_: Box::new(else_),
                }
            }
            RawExprKind::Identifier(ident) => {
                if let Some(var) = vars.get(&ident).cloned() {
                    Expr::Number(var)
                } else {
                    return err_ident!(ident);
                }
            }
            RawExprKind::Call { func, args } => Expr::Call {
                func,
                args: args
                    .into_iter()
                    .map(|raw_expr| raw_expr.resolve(vars, funcs))
                    .collect::<Result<Vec<Expr>, _>>()?,
            },

            RawExprKind::UserCall { name, args } => {
                // arity mismatch
                let func = funcs.get(&name).unwrap();
                let func_params = func.params();

                if func_params.len() != args.len() {
                    return err_fmt!(
                        "Resolver Error: function {} takes {} argument(s) but got {}",
                        func,
                        func_params.len(),
                        args.len()
                    );
                }

                Expr::UserCall {
                    name,
                    args: args
                        .into_iter()
                        .map(|raw_expr| raw_expr.resolve(vars, funcs))
                        .collect::<Result<Vec<Expr>, _>>()?,
                }
            }

            RawExprKind::Constant(constant) => Expr::Number(constant.value()),
        };

        Ok(expr)
    }
}
