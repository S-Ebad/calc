use crate::expression::{ExprKind, Expression};
use crate::lexer::Lexer;
use crate::resolver::resolver;
use crate::user_function::UserFunction;

use std::collections::HashMap;

const PRECISION: f64 = 1e10;

pub struct Calculator {
    pub variables: HashMap<String, f64>,
    pub funcs: HashMap<String, UserFunction>,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator {
            variables: HashMap::new(),
            funcs: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn set_user_function(&mut self, function: UserFunction) {
        self.funcs.insert(function.name.to_owned(), function);
    }

    pub fn solve(&mut self, buf: &str) -> Result<f64, String> {
        let mut lexer = Lexer::new(buf)?;

        let expr = Expression::parse(&mut lexer, &self.funcs)?;
        expr.check_errors()?;

        let ans = match expr.classify() {
            ExprKind::FuncDef(user_function) => {
                user_function.is_valid()?;
                self.set_user_function(user_function);

                return Ok(0.0);
            }

            ExprKind::Assign(name, mut expr) => {
                resolver(&mut expr, &self.variables, &self.funcs, 0)?;

                let ans = expr.eval()?;
                self.set_variable(&name, ans);

                ans
            }

            ExprKind::Eval(mut expr) => {
                resolver(&mut expr, &self.variables, &self.funcs, 0)?;

                expr.eval()?
            }
        };

        let ans = (ans * PRECISION).round() / PRECISION;
        self.set_variable("ans", ans);

        Ok(ans)
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}
