use std::{borrow::Borrow, collections::HashMap, fmt};

use crate::{
    calc::CacheKey,
    errors::{CalcError, Span},
    function::Function,
    operator::Operator,
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
    pub fn eval<K>(
        self,
        vars: &HashMap<K, f64>,
        funcs: &HashMap<String, UserFunction>,
        cache: &mut HashMap<CacheKey, f64>,
        depth: u32,
    ) -> Result<f64, CalcError>
    // temporary CalcError until we get EvalError/RuntimeError
    where
        K: Borrow<str>,
    {
        if depth >= 100 {
            return Err(CalcError::String(
                "Eval Error: recursion limit has been reached".to_string(),
            ));
        }

        match self.kind {
            ExprKind::Number(num) => Ok(num),
            ExprKind::Binary { op, lhs, rhs } => {
                let lhs = lhs.eval(vars, funcs, cache, depth)?;
                let rhs = rhs.eval(vars, funcs, cache, depth)?;

                op.perform_op(lhs, Some(rhs)).map_err(CalcError::String)
            }

            ExprKind::Unary { op, expr } | ExprKind::Postfix { op, expr } => op
                .perform_op(expr.eval(vars, funcs, cache, depth)?, None)
                .map_err(CalcError::String),

            ExprKind::Call { func, args } => {
                let args = args
                    .into_iter()
                    .map(|expr| expr.eval(vars, funcs, cache, depth))
                    .collect::<Result<Vec<f64>, _>>()?;

                func.call(&args).map_err(CalcError::String)
            }

            ExprKind::If {
                condition: condit,
                then,
                else_,
            } => {
                if condit.eval(vars, funcs, cache, depth)? != 0.0 {
                    then.eval(vars, funcs, cache, depth)
                } else {
                    else_.eval(vars, funcs, cache, depth)
                }
            }

            ExprKind::UserCall { name, args } => {
                let func = funcs.get(&name).unwrap();
                let func_params = func.params();

                let func_body = func.body().clone();

                let eval_args = args
                    .into_iter()
                    .map(|x| x.eval(vars, funcs, cache, depth))
                    .collect::<Result<Vec<f64>, _>>()?;

                let key = (name, to_bits(&eval_args));

                // cache hit
                if let Some(num) = cache.get(&key) {
                    return Ok(*num);
                }

                let mut scope = func_params
                    .iter()
                    .zip(eval_args)
                    .map(|(k, v)| (k.as_str(), v))
                    .collect::<HashMap<&str, f64>>();

                for (key, val) in vars.iter() {
                    scope.entry(key.borrow()).or_insert(*val);
                }

                // TODO: Later use EvalError::Resolution
                let new_body = func_body.resolve(&scope, funcs)?;
                let result = new_body.eval(&scope, funcs, cache, depth + 1)?;

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
