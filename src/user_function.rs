use std::{collections::HashMap, fmt};

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
        for param in self.params.iter() {
            if !matches!(param, Expression::Identifier(_)) {
                return Err(format!(
                    "Invalid Function Definition: parameter must be an identifier, got {}",
                    param
                ));
            }
        }

        Ok(())
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

        let mut vars: HashMap<String, f64> = HashMap::new();

        for (name, val) in self.params.into_iter().zip(args.iter()) {
            if let Expression::Identifier(name) = name {
                vars.insert(name, *val);
            }
        }

        Self::walk(&mut self.body, &vars);

        Ok(self.body)
    }

    fn walk(expr: &mut Expression, vars: &HashMap<String, f64>) {
        match expr {
            Expression::Number(_) => (),
            Expression::Constant(_) => (),

            Expression::Identifier(ident) => {
                // param
                if let Some(num) = vars.get(ident) {
                    *expr = Expression::Number(*num);
                }
            }

            Expression::Binary { op: _, lhs, rhs } => {
                Self::walk(rhs, vars);
                Self::walk(lhs, vars);
            }

            Expression::Unary { op: _, expr } | Expression::Postfix { expr, op: _ } => {
                Self::walk(expr, vars);
            }

            Expression::Apply {
                identifier: _,
                args,
            }
            | Expression::Call { func: _, args }
            | Expression::UserCall { func: _, args } => {
                args.iter_mut().for_each(|expr| Self::walk(expr, &vars))
            }
        }
    }
}

impl fmt::Display for UserFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
