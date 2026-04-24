use crate::expression::{Expression, IntoFunction};
use crate::lexer::Lexer;
use crate::resolver::resolver;

use std::collections::HashMap;

pub struct Calculator {
    pub variables: HashMap<String, f64>,
    pub funcs: HashMap<String, IntoFunction>,
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

    pub fn set_user_function(&mut self, function: IntoFunction) {
        self.funcs.insert(function.name.to_owned(), function);
    }

    pub fn solve(&mut self, buf: &str) -> Result<f64, String> {
        let mut lexer = Lexer::new(buf)?;
        let mut expr = Expression::parse(&mut lexer)?;

        if expr.is_func_def() {
            match expr.into_func() {
                Some(func) => {
                    println!("{:?}", &func);
                    self.set_user_function(func);

                    return Ok(0.0);
                }
                _ => unreachable!(),
            }
        }

        let ans;

        if expr.is_assign() {
            match expr.into_assign() {
                Some((name, mut expr)) => {
                    resolver(&mut expr, &self.variables, &self.funcs)?;
                    ans = expr.eval()?;

                    self.set_variable(&name, ans);
                }

                _ => unreachable!(),
            }
        } else {
            resolver(&mut expr, &self.variables, &self.funcs)?;

            ans = expr.eval()?;
        }

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
