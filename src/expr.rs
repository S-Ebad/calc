use std::{collections::HashMap, fmt};

use crate::{
    calc::CacheKey,
    errors::{EvalError, EvalErrorKind, Span},
    function::Function,
    operator::Operator,
    scope::Scope,
    user_function::UserFunction,
};

#[macro_export]
macro_rules! write_args {
    ($f:expr, $func:expr, $args:expr) => {
        write!(
            $f,
            "{}({})",
            $func,
            $args
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
}

fn to_bits(args: &[f64]) -> Vec<u64> {
    args.iter().map(|x| x.to_bits()).collect()
}

/// How far into a user call, and how much native stack that has cost.
///
/// `calls` bounds how deeply user functions may nest; it is the limit a
/// runaway recursion is expected to hit. `cost` approximates the stack itself,
/// which is a separate concern: resolving and evaluating a body happens
/// underneath the frames of every call above it, so a modest body depth at a
/// modest call depth still overflows.
///
/// Descents are weighted because they are not alike. The parser and the
/// resolver carry a whole match of locals per frame and run out of a default
/// thread stack at around a hundred and fifty levels; the evaluator survives
/// past three hundred. Counting them equally would either leave parsing
/// unprotected or refuse recursion depths that are perfectly safe.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Depth {
    calls: u32,
    cost: u32,
}

impl Depth {
    const MAX_CALLS: u32 = 80;

    // measured against the 2 MiB a spawned thread gets rather than the larger
    // main stack, since that is the smaller guarantee a caller has
    const MAX_COST: u32 = 320;
    const NESTING_COST: u32 = 4;
    const EVAL_COST: u32 = 1;

    pub fn new() -> Self {
        Self::default()
    }

    /// One level further into a nested expression, while parsing or resolving.
    pub fn nest(self) -> Self {
        Self {
            cost: self.cost + Self::NESTING_COST,
            ..self
        }
    }

    /// One level further down an expression tree, while evaluating.
    pub fn step(self) -> Self {
        Self {
            cost: self.cost + Self::EVAL_COST,
            ..self
        }
    }

    /// One level further into a user function body.
    pub fn call(self) -> Self {
        Self {
            calls: self.calls + 1,
            cost: self.cost + Self::EVAL_COST,
        }
    }

    pub fn calls_exceeded(&self) -> bool {
        self.calls >= Self::MAX_CALLS
    }

    pub fn exhausted(&self) -> bool {
        self.cost >= Self::MAX_COST
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {
    Number(f64),

    Binary {
        op: Operator,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Unary {
        op: Operator,
        expr: Box<Expr>,
    },

    Postfix {
        op: Operator,
        expr: Box<Expr>,
    },

    Call {
        func: Function,
        args: Vec<Expr>,
    },

    UserCall {
        name: String, //hold name only, definition is somewhere else
        args: Vec<Expr>,
    },

    If {
        condition: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub(crate) kind: ExprKind,
    pub(crate) span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn kind(&self) -> &ExprKind {
        &self.kind
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

impl std::ops::DerefMut for Expr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}

impl std::ops::Deref for Expr {
    type Target = ExprKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl Expr {
    pub fn eval(
        self,
        scope: &Scope,
        funcs: &HashMap<String, UserFunction>,
        cache: &mut HashMap<CacheKey, f64>,
        depth: Depth,
    ) -> Result<f64, EvalError> {
        // operator.rs and function.rs have no spans to offer; this is the layer
        // that knows where the expression came from, so it attaches them
        let span = self.span;
        let at_span = |kind| EvalError::new(kind, Some(span));

        if depth.calls_exceeded() {
            return Err(at_span(EvalErrorKind::RecursionLimit));
        }

        if depth.exhausted() {
            return Err(at_span(EvalErrorKind::ExpressionTooDeep));
        }

        let depth = depth.step();

        match self.kind {
            ExprKind::Number(num) => Ok(num),
            ExprKind::Binary { op, lhs, rhs } => {
                let lhs = lhs.eval(scope, funcs, cache, depth)?;
                let rhs = rhs.eval(scope, funcs, cache, depth)?;

                op.perform_op(lhs, Some(rhs)).map_err(at_span)
            }

            ExprKind::Unary { op, expr } | ExprKind::Postfix { op, expr } => op
                .perform_op(expr.eval(scope, funcs, cache, depth)?, None)
                .map_err(at_span),

            ExprKind::Call { func, args } => {
                let args = args
                    .into_iter()
                    .map(|expr| expr.eval(scope, funcs, cache, depth))
                    .collect::<Result<Vec<f64>, _>>()?;

                func.call(&args).map_err(at_span)
            }

            ExprKind::If {
                condition: condit,
                then,
                else_,
            } => {
                if condit.eval(scope, funcs, cache, depth)? != 0.0 {
                    then.eval(scope, funcs, cache, depth)
                } else {
                    else_.eval(scope, funcs, cache, depth)
                }
            }

            ExprKind::UserCall { name, args } => {
                let func = funcs.get(&name).unwrap();
                let func_params = func.params();

                // borrowed, not cloned: only the error path needs the source
                let func_src = func.src();

                let func_body = func.body().clone();

                let eval_args = args
                    .into_iter()
                    .map(|x| x.eval(scope, funcs, cache, depth))
                    .collect::<Result<Vec<f64>, _>>()?;

                let key = (name, to_bits(&eval_args));

                // cache hit
                if let Some(num) = cache.get(&key) {
                    return Ok(*num);
                }

                // the body's own parameters over the same globals
                let frame = scope.frame(func_params, &eval_args);

                // a name the callee could not find may be one of ours. that it
                // is out of reach is the deliberate rule rather than an
                // oversight, so the diagnostic should say which it is. every
                // boundary checks its own, so a name bound further up is still
                // recognised on the way out
                let explain = |err: EvalError| match err.unresolved_name() {
                    Some(unresolved) if scope.has_param(unresolved) => {
                        let note = format!(
                            "'{}' is a parameter of a calling function; pass it as an argument",
                            unresolved
                        );

                        err.or_note(note)
                    }

                    _ => err,
                };

                // under dynamic scoping the body can only be resolved once the
                // arguments are known, so resolution is an eval-time step here.
                // either step can fail inside the body, so both record the call
                let new_body = func_body.resolve(&frame, funcs, depth).map_err(|err| {
                    explain(EvalError::from(err)).push_frame(&key.0, func_src, span)
                })?;

                let result = new_body
                    .eval(&frame, funcs, cache, depth.call())
                    .map_err(|err| explain(err).push_frame(&key.0, func_src, span))?;

                cache.insert(key, result);

                Ok(result)
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{:.2}", n),

            Self::Binary { op, lhs, rhs } => write!(f, "{} {} {}", lhs, op, rhs),
            Self::Unary { op, expr } | Self::Postfix { op, expr } => {
                write!(f, "{}({})", op, expr)
            }
            Self::Call { func, args } => {
                write_args!(f, func, args)
            }
            Self::UserCall { name, args } => {
                write_args!(f, name, args)
            }
            Self::If {
                condition,
                then,
                else_,
            } => write!(f, "{} ? {} : {}", condition, then, else_),
        }
    }
}
