use std::fmt;

use crate::{
    errors::{ParseError, ParseErrorKind},
    raw_expr::{RawExpr, RawExprKind},
};

#[derive(Debug, Clone, PartialEq)]
pub struct UserFunction {
    name: String,
    params: Vec<String>,
    body: Box<RawExpr>,
    // the input line this was defined on. the body's spans index into it, so a
    // diagnostic raised inside the body needs this text to render against
    src: String,
}

impl UserFunction {
    pub fn new(
        name: String,
        params: Vec<RawExpr>,
        mut body: Box<RawExpr>,
        src: String,
    ) -> Result<Self, ParseError> {
        let params = params
            .into_iter()
            .map(|param| match param {
                RawExpr {
                    kind: RawExprKind::Identifier(name),
                    ..
                } => Ok(name),

                other => {
                    let span = other.span;
                    let kind = ParseErrorKind::InvalidParameter(other);

                    Err(ParseError::new(kind, Some(span)))
                }
            })
            .collect::<Result<Vec<String>, _>>()?;

        bind_params(&mut body, &params);

        Ok(Self {
            name,
            params,
            body,
            src,
        })
    }

    pub fn help(&self) -> String {
        // the definition as it was typed. reconstructing it from the body
        // instead would normalise the source away, turning '2x' into '2 * x'
        format!(
            "{} is a user-defined function.\n\n{}",
            self.name,
            self.src.trim()
        )
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &[String] {
        &self.params
    }

    pub fn arity(&self) -> usize {
        self.params.len()
    }

    pub fn body(&self) -> &RawExpr {
        &self.body
    }

    pub fn src(&self) -> &str {
        &self.src
    }
}

impl fmt::Display for UserFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Rewrite references to the function's own parameters into slot lookups.
///
/// Which parameter a name refers to is fixed the moment the function is
/// written, so the search can happen once here instead of on every call. Only
/// the function's own parameters are bound: anything else stays a name,
/// because dynamic scoping decides what it means at the point of the call.
fn bind_params(expr: &mut RawExpr, params: &[String]) {
    match &mut expr.kind {
        RawExprKind::Identifier(name) => {
            if let Some(slot) = params.iter().position(|param| param == name) {
                let name = std::rc::Rc::from(name.as_str());
                expr.kind = RawExprKind::Param { slot, name };
            }
        }

        RawExprKind::Binary { lhs, rhs, .. } => {
            bind_params(lhs, params);
            bind_params(rhs, params);
        }

        RawExprKind::Unary { expr, .. } | RawExprKind::Postfix { expr, .. } => {
            bind_params(expr, params)
        }

        RawExprKind::If {
            condition,
            then,
            else_,
        } => {
            bind_params(condition, params);
            bind_params(then, params);
            bind_params(else_, params);
        }

        RawExprKind::Apply { args, .. }
        | RawExprKind::Call { args, .. }
        | RawExprKind::UserCall { args, .. } => {
            for arg in args {
                bind_params(arg, params);
            }
        }

        RawExprKind::Number(_) | RawExprKind::Constant(_) | RawExprKind::Param { .. } => (),
    }
}
