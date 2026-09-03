use strum::IntoEnumIterator;

use crate::constant::Constant;
use crate::errors::{CalcError};
use crate::function::Function;
use crate::lexer::Lexer;
use crate::raw_expr::{RawExpr, Statement};
use crate::user_function::UserFunction;

use std::collections::HashMap;

const PRECISION: f64 = 1e10;

pub type CacheKey = (String, Vec<u64>);

pub struct Calculator {
    vars: HashMap<String, f64>,
    funcs: HashMap<String, UserFunction>,
    cache: HashMap<CacheKey, f64>,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            funcs: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, name: &str, value: f64) {
        // dynamic scoping screws with the cache because the variable could be in a function
        // then the cached result would actually be wrong
        if name != "ans" {
            self.cache.clear();
        }

        self.vars.insert(name.to_string(), value);
    }

    pub fn set_user_function(&mut self, function: UserFunction) {
        // clear cache for redefined functions
        self.cache.retain(|(name, _), _| name != function.name());

        self.funcs.insert(function.name().to_owned(), function);
    }

    fn help(&self, name: &str) {
        if name.is_empty() {
            let glob_funcs: Vec<String> = Function::iter()
                .map(|f| f.to_string().to_lowercase())
                .collect();

            let glob_const: Vec<String> = Constant::iter()
                .map(|c| c.to_string().to_lowercase())
                .collect();

            let user_funcs: Vec<&str> = self.funcs.keys().map(|k| k.as_str()).collect();

            println!("Available functions: {}", glob_funcs.join(", "));
            println!("Available constants: {}", glob_const.join(", "));

            if !user_funcs.is_empty() {
                println!(
                    "Available user-defined functions: {}",
                    user_funcs.join(", ")
                );
            }

            println!("\nType 'help <name>' for further details.");
            return;
        }

        let help_str = if let Some(func) = Function::from(name) {
            func.help()
        } else if let Some(user_func) = self.funcs.get(name) {
            &user_func.help()
        } else if let Some(constant) = Constant::from(name) {
            constant.help()
        } else {
            &format!(
                "No help found for '{}'. Type 'help' to see available functions and constants.",
                name
            )
        };

        println!("{}", help_str);
    }

    fn is_valid_help_name(s: &str) -> bool {
        s.is_empty() || s.chars().all(|c| c.is_alphabetic() || c == '_')
    }

    pub fn solve(&mut self, buf: &str) -> Result<Option<f64>, CalcError> {
        if let Some(rest) = buf.strip_prefix("help") {
            let rest = rest.trim();

            if Self::is_valid_help_name(rest) {
                self.help(rest);

                return Ok(None);
            }
        }

        let lexer = Lexer::new(buf)?;
        let expr = RawExpr::parse(lexer, &self.funcs)?;
        expr.check_errors()?;

        let stmt = expr.classify()?;

        let ans = match stmt {
            Statement::FuncDef(user_function) => {
                self.set_user_function(user_function);

                return Ok(None);
            }

            Statement::Assign(name, expr) => {
                let expr = expr.resolve(&self.vars, &self.funcs)?;

                let ans = expr.eval(&self.vars, &self.funcs, &mut self.cache, 0)?;
                self.set_variable(&name, ans);

                ans
            }

            Statement::Eval(expr) => {
                let expr = expr.resolve(&self.vars, &self.funcs)?;

                expr.eval(&self.vars, &self.funcs, &mut self.cache, 0)?
            }
        };

        let ans = (ans * PRECISION).round() / PRECISION;
        self.set_variable("ans", ans);

        Ok(Some(ans))
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}
