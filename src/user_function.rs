use std::fmt;

use crate::expression::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct UserFunction {
    pub name: String,
    pub params: Vec<String>, // guranteed to be [Identifier, ...]
    pub body: Box<Expression>,
}

impl UserFunction {
    pub fn new(name: String, params: Vec<Expression>, body: Box<Expression>) -> Self {
        Self { name, params, body }
    }

    pub fn is_valid(&self) -> Result<(), String> {
        self.params
            .iter()
            .find(|param| !matches!(param, Expression::Identifier(_)))
            .map(|param| {
                Err(format!(
                    "Invalid Function Definition: parameter must be an identifier, got '{}'",
                    param
                ))
            })
            .unwrap_or(Ok(()))
    }
}

impl fmt::Display for UserFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
