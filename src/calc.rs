use strum::IntoEnumIterator;

use crate::constant::Constant;
use crate::function::Function;
use crate::lexer::Lexer;
use crate::operator::Operator;
use crate::raw_expr::RawExpr;
use crate::user_function::UserFunction;

use std::collections::HashMap;

const PRECISION: f64 = 1e10;

pub type CacheKey = (String, Vec<u64>);

pub struct Calculator {
    vars: HashMap<String, f64>,
    funcs: HashMap<String, UserFunction>,
    cache: HashMap<CacheKey, f64>,
}

enum ExprKind {
    FuncDef(UserFunction),
    Assign(String, RawExpr),
    Eval(RawExpr),
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
            println!(
                "Available user-defined functions: {}",
                user_funcs.join(", ")
            );

            println!("Type 'help <name>' for further details.");

            return;
        }

        println!("Helping for: {}", name);
    }

    fn is_valid_help_name(s: &str) -> bool {
        s.is_empty() || s.chars().all(|c| c.is_alphabetic() || c == '_')
    }

    pub fn solve(&mut self, buf: &str) -> Result<Option<f64>, String> {
        if let Some(rest) = buf.strip_prefix("help") {
            let rest = rest.trim();

            if Self::is_valid_help_name(rest) {
                self.help(rest.trim());

                return Ok(None);
            }
        }

        let lexer = Lexer::new(buf)?;
        let expr = RawExpr::parse(lexer, &self.funcs)?;
        expr.check_errors()?;

        let ans = match Self::classify(expr)? {
            ExprKind::FuncDef(user_function) => {
                self.set_user_function(user_function);

                return Ok(None);
            }

            ExprKind::Assign(name, expr) => {
                let expr = expr.resolve(&self.vars, &self.funcs)?;

                let ans = expr.eval(&self.vars, &self.funcs, &mut self.cache, 0)?;
                self.set_variable(&name, ans);

                ans
            }

            ExprKind::Eval(expr) => {
                let expr = expr.resolve(&self.vars, &self.funcs)?;

                expr.eval(&self.vars, &self.funcs, &mut self.cache, 0)?
            }
        };

        let ans = (ans * PRECISION).round() / PRECISION;
        self.set_variable("ans", ans);

        // dbg!(&self.cache);
        Ok(Some(ans))
    }

    fn classify(expr: RawExpr) -> Result<ExprKind, String> {
        match expr {
            RawExpr::Binary {
                op: Operator::Equal,
                lhs,
                rhs,
            } => match *lhs {
                RawExpr::Apply { name, args } | RawExpr::UserCall { name, args } => {
                    Ok(ExprKind::FuncDef(UserFunction::new(name, args, rhs)?))
                }

                RawExpr::Identifier(ident) => Ok(ExprKind::Assign(ident, *rhs)),

                lhs => Ok(ExprKind::Eval(RawExpr::Binary {
                    op: Operator::Equal,
                    lhs: Box::new(lhs),
                    rhs,
                })),
            },

            other => Ok(ExprKind::Eval(other)),
        }
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}
