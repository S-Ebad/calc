use std::collections::HashMap;

use crate::{
    calc::CacheKey,
    errors::{EvalError, EvalErrorKind, Span},
    operator::Operator,
    raw_expr::RawExpr,
    resolver::Node,
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
/// which is a separate concern: a body is evaluated underneath the frames of
/// every call above it, so a modest body depth at a modest call depth still
/// overflows.
///
/// Descents are weighted because they are not alike: a level of parsing
/// carries a whole match of locals and takes more stack than a level of
/// evaluation. Counting them equally would either leave parsing unprotected or
/// refuse recursion depths that are perfectly safe.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Depth {
    calls: u32,
    cost: u32,
}

impl Depth {
    // an ordinary recursive body spends a few units of cost per call, so this
    // is reached well before the cost is, and a runaway recursion reports
    // itself as one
    const MAX_CALLS: u32 = 1000;

    // measured against `calc::STACK_SIZE` in an unoptimised build, which is
    // the larger per-level cost: the worst shapes measured spend just under
    // 4 KiB a unit, so this keeps about half of the stack in reserve
    const MAX_COST: u32 = 8000;
    const NESTING_COST: u32 = 4;
    const EVAL_COST: u32 = 1;

    pub fn new() -> Self {
        Self::default()
    }

    /// One level further into a nested expression, while parsing or checking.
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

impl RawExpr {
    /// Evaluate the expression where it is stored.
    ///
    /// Each node is resolved as it is reached rather than in a pass of its own,
    /// so a call reads its body in place: nothing is copied, a branch that is
    /// not taken is never looked at, and only one walk is ever on the stack.
    pub fn eval(
        &self,
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

        match self.resolve(scope, funcs)? {
            Node::Number(num) => Ok(num),

            Node::Binary { op, lhs, rhs } => {
                let lhs = lhs.eval(scope, funcs, cache, depth)?;
                let rhs = rhs.eval(scope, funcs, cache, depth)?;

                op.perform_op(lhs, Some(rhs)).map_err(at_span)
            }

            Node::Scaled { value, rhs } => {
                let rhs = rhs.eval(scope, funcs, cache, depth)?;

                Operator::ImplicitMul
                    .perform_op(value, Some(rhs))
                    .map_err(at_span)
            }

            Node::Unary { op, expr } => op
                .perform_op(expr.eval(scope, funcs, cache, depth)?, None)
                .map_err(at_span),

            Node::Call { func, args } => {
                let args = eval_args(args, scope, funcs, cache, depth)?;

                func.call(&args).map_err(at_span)
            }

            Node::If {
                condition,
                then,
                else_,
            } => {
                if condition.eval(scope, funcs, cache, depth)? != 0.0 {
                    then.eval(scope, funcs, cache, depth)
                } else {
                    else_.eval(scope, funcs, cache, depth)
                }
            }

            Node::UserCall { func, args } => {
                let args = eval_args(args, scope, funcs, cache, depth)?;

                call_user(func, args, span, scope, funcs, cache, depth)
            }
        }
    }
}

// the parts of a call live in functions of their own. an unoptimised build
// gives every local of a function its own slot, so kept inline they would sit
// in each frame of the walk rather than only while a call is being made

fn eval_args(
    args: &[RawExpr],
    scope: &Scope,
    funcs: &HashMap<String, UserFunction>,
    cache: &mut HashMap<CacheKey, f64>,
    depth: Depth,
) -> Result<Vec<f64>, EvalError> {
    args.iter()
        .map(|expr| expr.eval(scope, funcs, cache, depth))
        .collect()
}

fn call_user(
    func: &UserFunction,
    args: Vec<f64>,
    span: Span,
    scope: &Scope,
    funcs: &HashMap<String, UserFunction>,
    cache: &mut HashMap<CacheKey, f64>,
    depth: Depth,
) -> Result<f64, EvalError> {
    let key = (func.name().to_owned(), to_bits(&args));

    // cache hit
    if let Some(num) = cache.get(&key) {
        return Ok(*num);
    }

    // the body's own parameters over the same globals
    let frame = scope.frame(func.params(), &args);

    // a name the callee could not find may be one of ours. that it is out of
    // reach is the deliberate rule rather than an oversight, so the diagnostic
    // should say which it is. every boundary checks its own, so a name bound
    // further up is still recognised on the way out
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

    let result = func
        .body()
        .eval(&frame, funcs, cache, depth.call())
        .map_err(|err| explain(err).push_frame(func.name(), func.src(), span))?;

    cache.insert(key, result);

    Ok(result)
}
