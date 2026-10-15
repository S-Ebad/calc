use crate::{
    errors::{ResolverError, ResolverErrorKind},
    expr::{Expr, ExprKind},
    function::Function,
    operator::Operator,
    raw_expr::{RawExpr, RawExprKind},
    user_function::UserFunction,
};
use std::{borrow::Borrow, collections::HashMap, hash::Hash};

impl RawExpr {
    pub fn resolve<K>(
        self,
        vars: &HashMap<K, f64>,
        funcs: &HashMap<String, UserFunction>,
    ) -> Result<Expr, ResolverError>
    where
        K: Borrow<str> + Hash + Eq,
    {
        // carried onto the resolved node so eval errors can point at source too
        let span = self.span;

        let kind = match self.kind {
            RawExprKind::Number(n) => ExprKind::Number(n),
            RawExprKind::Binary { op, lhs, rhs } => {
                let lhs = lhs.resolve(vars, funcs)?;
                let rhs = rhs.resolve(vars, funcs)?;

                ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }
            RawExprKind::Unary { op, expr } => {
                let expr = expr.resolve(vars, funcs)?;

                ExprKind::Unary {
                    op,
                    expr: Box::new(expr),
                }
            }
            RawExprKind::Postfix { op, expr } => {
                let expr = expr.resolve(vars, funcs)?;

                ExprKind::Postfix {
                    op,
                    expr: Box::new(expr),
                }
            }
            RawExprKind::Apply { name, mut args } => {
                if let Some(var) = vars.get(&name).cloned() {
                    if args.len() != 1 {
                        let kind = ResolverErrorKind::InvalidMultiplication { target: name, args };
                        return Err(ResolverError::new(kind, Some(span)));
                    }

                    let arg = args.pop().unwrap();
                    let rhs = arg.resolve(vars, funcs)?;

                    let lhs = Expr::new(ExprKind::Number(var), span);

                    ExprKind::Binary {
                        op: Operator::ImplicitMul,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }
                } else if funcs.contains_key(&name) {
                    let resolved_args = args
                        .into_iter()
                        .map(|raw_expr| raw_expr.resolve(vars, funcs))
                        .collect::<Result<Vec<Expr>, _>>()?;

                    ExprKind::UserCall {
                        name,
                        args: resolved_args,
                    }
                } else if let Some(func) = Function::from(&name) {
                    let resolved_args = args
                        .into_iter()
                        .map(|raw_expr| raw_expr.resolve(vars, funcs))
                        .collect::<Result<Vec<_>, _>>()?;

                    ExprKind::Call {
                        func,
                        args: resolved_args,
                    }
                } else {
                    let kind = ResolverErrorKind::UnknownFunction(name);
                    return Err(ResolverError::new(kind, Some(span)));
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

                ExprKind::If {
                    condition: Box::new(condition),
                    then: Box::new(then),
                    else_: Box::new(else_),
                }
            }
            RawExprKind::Identifier(ident) => {
                if let Some(var) = vars.get(&ident).cloned() {
                    ExprKind::Number(var)
                } else {
                    let kind = if ident == "ans" {
                        ResolverErrorKind::UndefinedAns
                    } else {
                        ResolverErrorKind::UnknownIdentifier(ident)
                    };

                    return Err(ResolverError::new(kind, Some(span)));
                }
            }
            RawExprKind::Call { func, args } => ExprKind::Call {
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
                    let kind = ResolverErrorKind::ArityMismatch {
                        name: func.to_string(),
                        expected: func_params.len(),
                        got: args.len(),
                    };

                    return Err(ResolverError::new(kind, Some(span)));
                }

                ExprKind::UserCall {
                    name,
                    args: args
                        .into_iter()
                        .map(|raw_expr| raw_expr.resolve(vars, funcs))
                        .collect::<Result<Vec<Expr>, _>>()?,
                }
            }

            RawExprKind::Constant(constant) => ExprKind::Number(constant.value()),
        };

        Ok(Expr::new(kind, span))
    }
}
