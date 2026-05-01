use crate::expression::Expression;
use crate::lexer::Lexer;
use crate::resolver::resolver;
use crate::user_function::UserFunction;

use std::collections::HashMap;

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

        let mut expr = Expression::parse(&mut lexer, &self.funcs)?;
        expr.check_errors()?;

        if expr.is_func_def() {
            let func = expr.into_func().unwrap();

            func.is_valid()?;
            self.set_user_function(func);

            return Ok(0.0);
        }

        let ans = if expr.is_assign() {
            let (name, mut expr) = expr.into_assign().unwrap();

            resolver(&mut expr, &self.variables, &self.funcs, 0)?;

            let ans = expr.eval()?;
            self.set_variable(&name, ans);

            ans
        } else {
            resolver(&mut expr, &self.variables, &self.funcs, 0)?;

            expr.eval()?
        };

        let ans = (ans * 1e10).round() / 1e10;
        self.set_variable("ans", ans);

        Ok(ans)
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}
