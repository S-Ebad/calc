use std::collections::HashMap;

use crate::token::Token;

pub struct Calculator {
    pub variables: HashMap<String, f64>,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator {
            variables: HashMap::new(),
        }
    }

    pub fn add_variable(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn solve(&mut self, buf: &str) -> Result<f64, String> {
        let mut tokens = Self::tokenize(buf)?;

        let assign_to = if let [Token::Identifier(_), Token::Assign, ..] = tokens.as_slice() {
            let Token::Identifier(name) = tokens.remove(0) else {
                unreachable!()
            };

            tokens.remove(0); // remove assign

            Some(name)
        } else {
            None
        };

        let ans;
        {
            let result = self.parse(tokens).and_then(|rpn| self.eval(rpn))?;

            // truncate anything below 1e-10 (probably noise)
            ans = (result * 1e10).round() / 1e10
        }

        if let Some(name) = assign_to {
            self.add_variable(&name, ans);
        }

        self.add_variable("ans", ans);

        Ok(ans)
    }
}
