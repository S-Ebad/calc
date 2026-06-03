use std::fmt;

use crate::{constant::Constant, operator::Operator};

pub fn join(iter: impl Iterator<Item = impl ToString>, sep: &str) -> String {
    let mut s = String::new();

    for i in iter {
        s.push_str(&i.to_string());
        s.push_str(sep);
    }

    s
}

#[derive(Debug, PartialEq, Clone)]
pub enum RawExpr {
    Number(f64),
    Constant(Constant),

    Identifier(String),

    Binary {
        op: Operator,
        lhs: Box<RawExpr>,
        rhs: Box<RawExpr>,
    },

    Unary {
        op: Operator,
        expr: Box<RawExpr>,
    },

    Postfix {
        op: Operator,
        expr: Box<RawExpr>,
    },

    // before its resolved
    Apply {
        name: String,
        args: Vec<RawExpr>,
    },

    If {
        condition: Box<RawExpr>,
        then: Box<RawExpr>,
        else_: Box<RawExpr>,
    },
}

impl fmt::Display for RawExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        match self {
            RawExpr::Number(n) => write!(f, "{:.2}", n),
            RawExpr::Constant(constant) => write!(f, "{}", constant),
            RawExpr::Identifier(ident) => write!(f, "{}", ident),
            RawExpr::Binary { op, lhs, rhs } => write!(f, "{}(lhs={}, lhs={})", op, lhs, rhs),
            RawExpr::Unary { op, expr } | RawExpr::Postfix { op, expr } => write!(f, "{}({})", op, expr),
            RawExpr::Apply { name, args } => write!(f, "Apply({}, args={})", name, join(args.iter(), ", ")),
            RawExpr::If { condition, then, else_ } => write!(f, "If({}, then={}, else={})", condition, then, else_),
        }
    }
}
