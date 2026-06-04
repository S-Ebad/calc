use std::{borrow::Borrow, collections::HashMap, fmt};

use crate::{function::Function, operator::Operator, user_function::UserFunction};

#[macro_export]
macro_rules! write_args {
    ($f:expr, $name:literal, $func:expr, $args:expr) => {
        write!(
            $f,
            "{}({}, args=[{}])",
            $name,
            $func,
            $args
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
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

impl Expr {
    pub fn eval<K>(
        self,
        vars: &HashMap<K, f64>,
        funcs: &HashMap<String, UserFunction>,
        depth: u32,
    ) -> Result<f64, String>
    where
        K: Borrow<str>,
    {

        if depth >= 100 {
            return Err("Evaluation Error: Recursion limit has been reached".to_string())
        }

        match self {
            Expr::Number(num) => Ok(num),
            Expr::Binary { op, lhs, rhs } => {
                let lhs = lhs.eval(vars, funcs, depth)?;
                let rhs = rhs.eval(vars, funcs, depth)?;

                op.perform_op(lhs, Some(rhs))
            }
            Expr::Unary { op, expr } => op.perform_op(expr.eval(vars, funcs, depth)?, None),
            Expr::Postfix { expr, op } => op.perform_op(expr.eval(vars, funcs, depth)?, None),
            Expr::Call { func, args } => {
                let args = args
                    .into_iter()
                    .map(|expr| expr.eval(vars, funcs, depth))
                    .collect::<Result<Vec<f64>, _>>()?;

                func.call(&args)
            }

            Expr::If {
                condition: condit,
                then,
                else_,
            } => {
                if condit.eval(vars, funcs, depth)? != 0.0 {
                    then.eval(vars, funcs, depth)
                } else {
                    else_.eval(vars, funcs, depth)
                }
            }

            Expr::UserCall { name, args } => {
                // arity mismatch
                let func = funcs.get(&name).unwrap();
                let func_params = func.params();

                let func_body = func.body().clone();

                let eval_args = args
                    .into_iter()
                    .map(|x| x.eval(vars, funcs, depth))
                    .collect::<Result<Vec<f64>, _>>()?;

                let mut scope = func_params
                    .iter()
                    .zip(eval_args)
                    .map(|(k, v)| (k.as_str(), v))
                    .collect::<HashMap<&str, f64>>();

                for (key, val) in vars.iter() {
                    scope.entry(key.borrow()).or_insert(*val);
                }

                let new_body = func_body.clone().resolve(&scope, funcs)?;
                new_body.eval(&scope, funcs, depth+1)
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(n) => write!(f, "{:.2}", n),

            Expr::Binary { op, lhs, rhs } => write!(f, "{}(lhs={}, lhs={})", op, lhs, rhs),
            Expr::Unary { op, expr } | Expr::Postfix { op, expr } => {
                write!(f, "{}({})", op, expr)
            }
            Expr::Call { func, args } => {
                write_args!(f, "Call", func, args)
            }
            Expr::UserCall { name, args } => {
                write_args!(f, "UserCall", name, args)
            }
            Expr::If {
                condition,
                then,
                else_,
            } => write!(f, "If({}, then={}, else={})", condition, then, else_),
        }
    }
}
