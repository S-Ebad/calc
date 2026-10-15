use crate::{
    errors::{ResolverError, ResolverErrorKind},
    expr::{Depth, Expr, ExprKind},
    function::Function,
    operator::Operator,
    raw_expr::{RawExpr, RawExprKind},
    scope::Scope,
    user_function::UserFunction,
};
use std::collections::HashMap;

impl RawExpr {
    pub fn resolve(
        self,
        scope: &Scope,
        funcs: &HashMap<String, UserFunction>,
        depth: Depth,
    ) -> Result<Expr, ResolverError> {
        // carried onto the resolved node so eval errors can point at source too
        let span = self.span;

        // resolve descends the tree, so it grows the native stack the same way
        // eval does, and under dynamic scoping it runs beneath eval's frames
        if depth.nodes_exceeded() {
            let kind = ResolverErrorKind::ExpressionTooDeep;
            return Err(ResolverError::new(kind, Some(span)));
        }

        let depth = depth.descend();

        let kind = match self.kind {
            RawExprKind::Number(n) => ExprKind::Number(n),
            RawExprKind::Binary { op, lhs, rhs } => {
                let lhs = lhs.resolve(scope, funcs, depth)?;
                let rhs = rhs.resolve(scope, funcs, depth)?;

                ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }
            RawExprKind::Unary { op, expr } => {
                let expr = expr.resolve(scope, funcs, depth)?;

                ExprKind::Unary {
                    op,
                    expr: Box::new(expr),
                }
            }
            RawExprKind::Postfix { op, expr } => {
                let expr = expr.resolve(scope, funcs, depth)?;

                ExprKind::Postfix {
                    op,
                    expr: Box::new(expr),
                }
            }
            RawExprKind::Apply { name, mut args } => {
                if let Some(var) = scope.get(&name) {
                    if args.len() != 1 {
                        let kind = ResolverErrorKind::InvalidMultiplication { target: name, args };
                        return Err(ResolverError::new(kind, Some(span)));
                    }

                    let arg = args.pop().unwrap();
                    let rhs = arg.resolve(scope, funcs, depth)?;

                    let lhs = Expr::new(ExprKind::Number(var), span);

                    ExprKind::Binary {
                        op: Operator::ImplicitMul,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }
                } else if let Some(user_func) = funcs.get(&name) {
                    if user_func.arity() != args.len() {
                        let kind = ResolverErrorKind::ArityMismatch {
                            name: user_func.to_string(),
                            expected: user_func.arity(),
                            got: args.len(),
                        };

                        return Err(ResolverError::new(kind, Some(span)));
                    }

                    let resolved_args = args
                        .into_iter()
                        .map(|raw_expr| raw_expr.resolve(scope, funcs, depth))
                        .collect::<Result<Vec<Expr>, _>>()?;

                    ExprKind::UserCall {
                        name,
                        args: resolved_args,
                    }
                } else if let Some(func) = Function::from(&name) {
                    let resolved_args = args
                        .into_iter()
                        .map(|raw_expr| raw_expr.resolve(scope, funcs, depth))
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
                let condition = condition.resolve(scope, funcs, depth)?;
                let then = then.resolve(scope, funcs, depth)?;
                let else_ = else_.resolve(scope, funcs, depth)?;

                ExprKind::If {
                    condition: Box::new(condition),
                    then: Box::new(then),
                    else_: Box::new(else_),
                }
            }
            RawExprKind::Identifier(ident) => {
                if let Some(var) = scope.get(&ident) {
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
                    .map(|raw_expr| raw_expr.resolve(scope, funcs, depth))
                    .collect::<Result<Vec<Expr>, _>>()?,
            },

            RawExprKind::UserCall { name, args } => {
                // arity mismatch
                let func = funcs.get(&name).unwrap();

                if func.arity() != args.len() {
                    let kind = ResolverErrorKind::ArityMismatch {
                        name: func.to_string(),
                        expected: func.arity(),
                        got: args.len(),
                    };

                    return Err(ResolverError::new(kind, Some(span)));
                }

                ExprKind::UserCall {
                    name,
                    args: args
                        .into_iter()
                        .map(|raw_expr| raw_expr.resolve(scope, funcs, depth))
                        .collect::<Result<Vec<Expr>, _>>()?,
                }
            }

            RawExprKind::Param { slot, name } => {
                let Some(arg) = scope.param(slot) else {
                    let kind = ResolverErrorKind::UnboundParameter(name);
                    return Err(ResolverError::new(kind, Some(span)));
                };

                ExprKind::Number(arg)
            }

            RawExprKind::Constant(constant) => ExprKind::Number(constant.value()),
        };

        Ok(Expr::new(kind, span))
    }
}
