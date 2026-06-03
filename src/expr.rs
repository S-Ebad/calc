use crate::{function::Function, operator::Operator};

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
        func: String, //hold name only, reference is somewhere else
        args: Vec<Expr>,
    },

    If {
        condition: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
}
