use crate::expression::Expression;
use crate::lexer::Lexer;
use crate::resolver::resolver;

use std::collections::HashMap;

pub struct Calculator {
    pub variables: HashMap<String, f64>,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator {
            variables: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn solve(&mut self, buf: &str) -> Result<f64, String> {
        let mut lexer = Lexer::new(buf)?;
        let mut expr = Expression::parse(&mut lexer)?;

        resolver(&mut expr, &self.variables)?;

        let ans = (expr.eval()? * 1e10).round() / 1e10;

        self.set_variable("ans", ans);

        Ok(ans)
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}
