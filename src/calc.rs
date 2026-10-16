use strum::IntoEnumIterator;

use crate::constant::Constant;
use crate::errors::{CalcError, ResolverError, ResolverErrorKind, Span};
use crate::expr::Depth;
use crate::function::Function;
use crate::lexer::Lexer;
use crate::raw_expr::{RawExpr, Statement};
use crate::scope::Scope;
use crate::user_function::UserFunction;

use std::collections::HashMap;

const PRECISION: f64 = 1e10;

/// The stack every line is solved on, which the limits in `Depth` are
/// measured against.
pub const STACK_SIZE: usize = 64 * 1024 * 1024;

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

    /// A name stands for one kind of thing. Letting it be both a variable and
    /// a function leaves one of them unreachable, since whether a name is a
    /// call is decided while the line is still being read.
    ///
    /// Replacing a name with another of its own kind is fine; only crossing
    /// from one kind to the other is refused.
    fn check_free_for(
        &self,
        taken: bool,
        kind: ResolverErrorKind,
        span: Span,
    ) -> Result<(), ResolverError> {
        if !taken {
            return Ok(());
        }

        Err(ResolverError::new(kind, Some(span)))
    }

    fn check_not_a_function(&self, name: &str, span: Span) -> Result<(), ResolverError> {
        let taken = self.funcs.contains_key(name);
        let kind = ResolverErrorKind::AlreadyAFunction(name.to_owned());

        self.check_free_for(taken, kind, span)
    }

    fn check_not_a_variable(&self, name: &str, span: Span) -> Result<(), ResolverError> {
        let taken = self.vars.contains_key(name);
        let kind = ResolverErrorKind::AlreadyAVariable(name.to_owned());

        self.check_free_for(taken, kind, span)
    }

    pub fn set_variable(&mut self, name: &str, value: f64) {
        // a function body reads a global when it is called, not when it is
        // written, so a cached result may depend on this name without the key
        // saying so. only the functions that actually read it are at risk,
        // which is what keeps 'ans' from clearing the cache after every answer
        if self.funcs.values().any(|func| func.reads(name)) {
            self.cache.clear();
        }

        self.vars.insert(name.to_string(), value);
    }

    pub fn set_user_function(&mut self, function: UserFunction) {
        // not just this function's own entries: anything that called it cached
        // a result computed from the body being replaced
        self.cache.clear();

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

    /// What a line is asking for help with, if it is asking at all.
    ///
    /// The word has to stand on its own. Letters running straight on from it
    /// are one identifier, so 'helpme' is a name like any other rather than a
    /// request for help with 'me'.
    fn help_argument(buf: &str) -> Option<&str> {
        let line = buf.trim();
        let rest = line.strip_prefix("help")?;

        if rest.is_empty() {
            return Some(rest);
        }

        rest.starts_with(char::is_whitespace).then(|| rest.trim())
    }

    fn is_valid_help_name(s: &str) -> bool {
        s.is_empty() || s.chars().all(|c| c.is_alphabetic() || c == '_')
    }

    /// Evaluate one line of input.
    ///
    /// The work runs on a thread of its own, so how deep an expression or a
    /// recursion may go is set by a stack this crate chose rather than by
    /// whatever thread the caller happens to be on. The stack is only reserved:
    /// what is actually used grows with the depth reached.
    pub fn solve(&mut self, buf: &str) -> Result<Option<f64>, CalcError> {
        std::thread::scope(|scope| {
            let worker = std::thread::Builder::new()
                .name("calc".to_string())
                .stack_size(STACK_SIZE)
                .spawn_scoped(scope, || self.solve_on_this_thread(buf))
                .expect("failed to spawn the evaluation thread");

            // a panic is a bug rather than a result, so it carries on as one
            worker
                .join()
                .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
        })
    }

    /// Solve each statement of the input in turn, separated by ';'.
    ///
    /// Statements run in order, so a later one sees what an earlier one
    /// defined, and the result is the last statement's. The first error stops
    /// the rest; what ran before it stays done.
    fn solve_on_this_thread(&mut self, buf: &str) -> Result<Option<f64>, CalcError> {
        let separated = buf.contains(';');

        let mut ans = None;
        let mut offset = 0;

        for statement in buf.split(';') {
            // a trailing ';' or a doubled one leaves nothing to solve. an
            // input that is empty without any separator is still a mistake
            if !(separated && statement.trim().is_empty()) {
                // each statement is solved against its own text, so a function
                // keeps only its own definition rather than the whole input,
                // and the error is moved back to where it sits in the input
                ans = self
                    .solve_statement(statement)
                    .map_err(|err| err.shifted(offset))?;
            }

            offset += statement.len() + ';'.len_utf8();
        }

        Ok(ans)
    }

    fn solve_statement(&mut self, buf: &str) -> Result<Option<f64>, CalcError> {
        // whitespace around a statement is only what separates it from the
        // one before
        if let Some(rest) =
            Self::help_argument(buf.trim()).filter(|rest| Self::is_valid_help_name(rest))
        {
            self.help(rest);

            return Ok(None);
        }

        let lexer = Lexer::new(buf)?;
        let expr = RawExpr::parse(lexer, &self.funcs)?;
        expr.check_errors()?;

        let stmt = expr.classify(buf)?;

        let ans = match stmt {
            Statement::FuncDef(user_function, head) => {
                self.check_not_a_variable(user_function.name(), head)?;
                user_function.body().check_builtin_calls()?;
                self.set_user_function(user_function);

                return Ok(None);
            }

            Statement::Assign(name, head, expr) => {
                self.check_not_a_function(&name, head)?;

                expr.check(&Scope::globals(&self.vars), &self.funcs, Depth::new())?;

                let ans = expr.eval(
                    &Scope::globals(&self.vars),
                    &self.funcs,
                    &mut self.cache,
                    Depth::new(),
                )?;
                self.set_variable(&name, ans);

                ans
            }

            Statement::Eval(expr) => {
                expr.check(&Scope::globals(&self.vars), &self.funcs, Depth::new())?;

                expr.eval(
                    &Scope::globals(&self.vars),
                    &self.funcs,
                    &mut self.cache,
                    Depth::new(),
                )?
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
