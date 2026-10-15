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
        body: Box<RawExpr>,
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
