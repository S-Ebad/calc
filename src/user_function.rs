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
}

impl UserFunction {
    pub fn new(name: String, params: Vec<RawExpr>, body: Box<RawExpr>) -> Result<Self, ParseError> {
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

        Ok(Self { name, params, body })
    }

    pub fn help(&self) -> String {
        format!(
            "{} is a user-defined function.\n\n{}({}) = {}",
            self.name,
            self.name,
            self.params.join(", "),
            self.body.kind
        )
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &[String] {
        &self.params
    }

    pub fn body(&self) -> &RawExpr {
        &self.body
    }
}

impl fmt::Display for UserFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
