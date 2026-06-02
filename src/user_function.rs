use std::{f64, fmt};

use crate::expression::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct UserFunction {
    pub name: String,
    pub params: Vec<Expression>, // guranteed to be [Identifier, ...]
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

    pub fn inline(mut self, args: &[f64]) -> Result<Box<Expression>, String> {
        // arity check
        if self.params.len() != args.len() {
            return Err(format!(
                "Invalid Arguments: Function {} takes {} argument(s) but got {}",
                self,
                self.params.len(),
                args.len()
            ));
        }

        let vars: Vec<(String, f64)> = self
            .params
            .into_iter()
            .zip(args.iter())
            .filter_map(|(name, val)| match name {
                Expression::Identifier(name) => Some((name, *val)),
                _ => None,
            })
            .collect();

        Self::walk(&mut self.body, &vars);

        Ok(self.body)
    }

    fn get_var<'a>(vars: &'a [(String, f64)], target: &str) -> Option<&'a f64> {
        vars.iter().find(|(name, _)| name == target).map(|(_, v)| v)
    }

    fn walk(expr: &mut Expression, vars: &[(String, f64)]) {
        match expr {
            Expression::Number(_) => (),
            Expression::Constant(_) => (),

            Expression::Identifier(ident) => {
                // param
                if let Some(num) =Self::get_var(vars, ident) {
                    *expr = Expression::Number(*num);
                }
            }

            Expression::Binary { op: _, lhs, rhs } => {
                Self::walk(rhs, vars);
                Self::walk(lhs, vars);
            }

            Expression::Unary { op: _, expr } | Expression::Postfix { expr, .. } => {
                Self::walk(expr, vars);
            }

            Expression::Apply { identifier, args } => {
                args.iter_mut().for_each(|expr| Self::walk(expr, vars));

                if let Some(num) = Self::get_var(vars, identifier) {
                    *expr = Expression::Number(*num);
                }
            }

            Expression::Call { func: _, args } | Expression::UserCall { func: _, args } => {
                args.iter_mut().for_each(|expr| Self::walk(expr, vars))
            }

            Expression::If { condition, then, else_ } => {
                Self::walk(condition, vars);
                Self::walk(then, vars);
                Self::walk(else_, vars);
            },
        }
    }
}

impl fmt::Display for UserFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
