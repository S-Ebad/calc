use crate::lexer::Lexer;
use crate::operator::Operator;
use crate::raw_expr::RawExpr;
use crate::user_function::UserFunction;

use std::collections::HashMap;

const PRECISION: f64 = 1e10;

pub struct Calculator {
    variables: HashMap<String, f64>,
    funcs: HashMap<String, UserFunction>,
}

enum ExprKind {
    FuncDef(UserFunction),
    Assign(String, RawExpr),
    Eval(RawExpr),
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
        self.funcs.insert(function.name().to_owned(), function);
    }

    pub fn solve(&mut self, buf: &str) -> Result<Option<f64>, String> {
        let lexer = Lexer::new(buf)?;
        let expr = RawExpr::parse(lexer, &self.funcs)?;
        expr.check_errors()?;

        let ans = match Self::classify(expr)? {
            ExprKind::FuncDef(user_function) => {
                self.set_user_function(user_function);

                return Ok(None);
            }

            ExprKind::Assign(name, expr) => {
                let expr = expr.resolve(&self.variables, &self.funcs)?;

                let ans = expr.eval(&self.variables, &self.funcs, 0)?;
                self.set_variable(&name, ans);

                ans
            }

            ExprKind::Eval(expr) => {
                let expr = expr.resolve(&self.variables, &self.funcs)?;

                expr.eval(&self.variables, &self.funcs, 0)?
            }
        };

        let ans = (ans * PRECISION).round() / PRECISION;
        self.set_variable("ans", ans);

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
