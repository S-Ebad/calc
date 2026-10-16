use crate::{
    errors::{ResolverError, ResolverErrorKind, Span},
    expr::Depth,
    function::Function,
    operator::Operator,
    raw_expr::{RawExpr, RawExprKind},
    scope::Scope,
    user_function::UserFunction,
};
use std::collections::HashMap;

/// What a single node means in a given scope, with its children left as they
/// were parsed.
///
/// Resolving one node at a time is what lets a body be evaluated in place: the
/// children are borrowed from the stored definition, so a call neither copies
/// the body nor walks a branch it is not going to take.
pub enum Node<'a> {
    Number(f64),

    Binary {
        op: Operator,
        lhs: &'a RawExpr,
        rhs: &'a RawExpr,
    },

    /// a variable written next to a parenthesised value, as in 'a(2)'
    Scaled {
        value: f64,
        rhs: &'a RawExpr,
    },

    Unary {
        op: Operator,
        expr: &'a RawExpr,
    },

    Call {
        func: Function,
        args: &'a [RawExpr],
    },

    UserCall {
        func: &'a UserFunction,
        args: &'a [RawExpr],
    },

    If {
        condition: &'a RawExpr,
        then: &'a RawExpr,
        else_: &'a RawExpr,
    },
}

impl RawExpr {
    /// Decide what this node refers to, without looking below it.
    pub fn resolve<'a>(
        &'a self,
        scope: &Scope,
        funcs: &'a HashMap<String, UserFunction>,
    ) -> Result<Node<'a>, ResolverError> {
        let span = self.span;

        let node = match &self.kind {
            RawExprKind::Number(n) => Node::Number(*n),
            RawExprKind::Constant(constant) => Node::Number(constant.value()),

            RawExprKind::Binary { op, lhs, rhs } => Node::Binary { op: *op, lhs, rhs },

            RawExprKind::Unary { op, expr } | RawExprKind::Postfix { op, expr } => {
                Node::Unary { op: *op, expr }
            }

            RawExprKind::Apply { name, args } => {
                if let Some(value) = scope.get(name) {
                    let [rhs] = args.as_slice() else {
                        let kind = ResolverErrorKind::InvalidMultiplication {
                            target: name.clone(),
                            args: args.clone(),
                        };

                        return Err(ResolverError::new(kind, Some(span)));
                    };

                    Node::Scaled { value, rhs }
                } else if let Some(user_func) = funcs.get(name) {
                    user_call(user_func, args, span)?
                } else if let Some(func) = Function::from(name) {
                    check_builtin_arity(func, args.len(), span)?;

                    Node::Call { func, args }
                } else {
                    let kind = ResolverErrorKind::UnknownFunction(name.clone());
                    return Err(ResolverError::new(kind, Some(span)));
                }
            }

            RawExprKind::Method { name, args } => {
                // the function wins over a variable of the same name, since
                // '.' only ever calls. parameters never get here: the parser
                // already refused them
                if let Some(user_func) = funcs.get(name) {
                    user_call(user_func, args, span)?
                } else if scope.get(name).is_some() {
                    let note = "only functions can be called with '.'".to_string();
                    let kind = ResolverErrorKind::NotAFunction(name.clone());

                    return Err(ResolverError::with_note(kind, Some(span), note));
                } else {
                    let kind = ResolverErrorKind::UnknownFunction(name.clone());
                    return Err(ResolverError::new(kind, Some(span)));
                }
            }

            RawExprKind::If {
                condition,
                then,
                else_,
            } => Node::If {
                condition,
                then,
                else_,
            },

            RawExprKind::Identifier(ident) => {
                if let Some(value) = scope.get(ident) {
                    Node::Number(value)
                } else {
                    let kind = if ident == "ans" {
                        ResolverErrorKind::UndefinedAns
                    } else {
                        ResolverErrorKind::UnknownIdentifier(ident.clone())
                    };

                    return Err(ResolverError::new(kind, Some(span)));
                }
            }

            RawExprKind::Call { func, args } => {
                check_builtin_arity(*func, args.len(), span)?;

                Node::Call { func: *func, args }
            }

            RawExprKind::UserCall { name, args } => user_call(&funcs[name], args, span)?,

            RawExprKind::Param { slot, name } => {
                let Some(arg) = scope.param(*slot) else {
                    let kind = ResolverErrorKind::UnboundParameter(name.clone());
                    return Err(ResolverError::new(kind, Some(span)));
                };

                Node::Number(arg)
            }
        };

        Ok(node)
    }

    /// Resolve every node of the expression without evaluating any of it.
    ///
    /// Only an expression typed at the prompt is checked this way, so a
    /// mistake in a branch that is never taken is still reported. A body is not:
    /// its names are only known at the moment of the call, and then only the
    /// branch actually taken matters.
    pub fn check(
        &self,
        scope: &Scope,
        funcs: &HashMap<String, UserFunction>,
        depth: Depth,
    ) -> Result<(), ResolverError> {
        if depth.exhausted() {
            let kind = ResolverErrorKind::ExpressionTooDeep;
            return Err(ResolverError::new(kind, Some(self.span)));
        }

        let depth = depth.nest();

        match self.resolve(scope, funcs)? {
            Node::Number(_) => Ok(()),

            Node::Binary { lhs, rhs, .. } => {
                lhs.check(scope, funcs, depth)?;
                rhs.check(scope, funcs, depth)
            }

            Node::Scaled { rhs: expr, .. } | Node::Unary { expr, .. } => {
                expr.check(scope, funcs, depth)
            }

            Node::Call { args, .. } | Node::UserCall { args, .. } => args
                .iter()
                .try_for_each(|arg| arg.check(scope, funcs, depth)),

            Node::If {
                condition,
                then,
                else_,
            } => {
                condition.check(scope, funcs, depth)?;
                then.check(scope, funcs, depth)?;
                else_.check(scope, funcs, depth)
            }
        }
    }

    /// Reject the mistakes in a body that no later definition could fix.
    ///
    /// How many arguments a built-in takes never changes, so a wrong count is
    /// reported where the function is written instead of on whichever call
    /// first happens to take that branch.
    pub fn check_builtin_calls(&self) -> Result<(), ResolverError> {
        match &self.kind {
            RawExprKind::Call { func, args } => {
                check_builtin_arity(*func, args.len(), self.span)?;
                args.iter().try_for_each(RawExpr::check_builtin_calls)
            }

            RawExprKind::Apply { args, .. }
            | RawExprKind::Method { args, .. }
            | RawExprKind::UserCall { args, .. } => {
                args.iter().try_for_each(RawExpr::check_builtin_calls)
            }

            RawExprKind::Binary { lhs, rhs, .. } => {
                lhs.check_builtin_calls()?;
                rhs.check_builtin_calls()
            }

            RawExprKind::Unary { expr, .. } | RawExprKind::Postfix { expr, .. } => {
                expr.check_builtin_calls()
            }

            RawExprKind::If {
                condition,
                then,
                else_,
            } => {
                condition.check_builtin_calls()?;
                then.check_builtin_calls()?;
                else_.check_builtin_calls()
            }

            RawExprKind::Number(_)
            | RawExprKind::Constant(_)
            | RawExprKind::Identifier(_)
            | RawExprKind::Param { .. } => Ok(()),
        }
    }
}

/// A call to a user function, once the name is known to be one.
fn user_call<'a>(
    user_func: &'a UserFunction,
    args: &'a [RawExpr],
    span: Span,
) -> Result<Node<'a>, ResolverError> {
    if user_func.arity() != args.len() {
        let kind = ResolverErrorKind::ArityMismatch {
            name: user_func.to_string(),
            expected: user_func.arity(),
            got: args.len(),
        };

        return Err(ResolverError::new(kind, Some(span)));
    }

    Ok(Node::UserCall {
        func: user_func,
        args,
    })
}

/// How many arguments a built-in takes is fixed by which built-in it is, so a
/// wrong count is settled without evaluating anything.
fn check_builtin_arity(func: Function, got: usize, span: Span) -> Result<(), ResolverError> {
    let (min, max) = func.arity();

    if (min..=max).contains(&got) {
        return Ok(());
    }

    let kind = ResolverErrorKind::BuiltinArity { func, got };

    Err(ResolverError::new(kind, Some(span)))
}
