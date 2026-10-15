//! Shared test helpers.
//!
//! Each file in `tests/` is its own crate, so anything used by more than one
//! of them lives here and is pulled in with `mod common;`. The directory keeps
//! it from being collected as a test binary of its own.

// every file pulls the whole module in, and none of them use all of it
#![allow(dead_code)]

pub use calc::calc::Calculator;
pub use calc::errors::{
    CalcError, EvalErrorKind, LexerErrorKind, ParseErrorKind, ResolverErrorKind, render_error,
};

/// A calculator that keeps its state across several lines, the way the REPL
/// does. Variables, definitions and the answer cache all persist.
pub struct Session {
    calc: Calculator,
}

impl Session {
    pub fn new() -> Self {
        Self {
            calc: Calculator::new(),
        }
    }

    /// Set-up that has to succeed. Returns self so definitions can be chained.
    pub fn ok(&mut self, line: &str) -> &mut Self {
        if let Err(err) = self.calc.solve(line) {
            panic!("{}", render_error(line, err));
        }

        self
    }

    /// A line that has to produce a value.
    pub fn value(&mut self, line: &str) -> f64 {
        match self.calc.solve(line) {
            Ok(Some(value)) => value,
            Ok(None) => panic!("{:?} produced no value", line),
            Err(err) => panic!("{}", render_error(line, err)),
        }
    }

    /// A line that has to fail, giving back the error for inspection.
    pub fn err(&mut self, line: &str) -> CalcError {
        match self.calc.solve(line) {
            Err(err) => err,
            Ok(value) => panic!("{:?} unexpectedly succeeded with {:?}", line, value),
        }
    }

    /// A line that has to fail, rendered the way the REPL would print it.
    pub fn render(&mut self, line: &str) -> String {
        let err = self.err(line);
        render_error(line, err)
    }

    pub fn is_err(&mut self, line: &str) -> bool {
        self.calc.solve(line).is_err()
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluate one expression on its own.
pub fn solve(expr: &str) -> f64 {
    Session::new().value(expr)
}

pub fn solve_is_err(expr: &str) -> bool {
    Session::new().is_err(expr)
}

/// The error from an expression that has to fail.
pub fn err(expr: &str) -> CalcError {
    Session::new().err(expr)
}

/// The rendered diagnostic for an expression that has to fail.
pub fn render(expr: &str) -> String {
    Session::new().render(expr)
}

/// Trim the floating point noise the calculator itself trims.
pub fn round(n: f64) -> f64 {
    (n * 1e10).round() / 1e10
}

pub fn approx(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {}, got {}",
        expected,
        actual
    );
}

// Which stage produced an error, and what it was. The fields are private, so
// a test cannot reach a kind without going through one of these.

pub fn lexer_kind(err: &CalcError) -> Option<&LexerErrorKind> {
    match err {
        CalcError::Lexer(err) => Some(err.kind()),
        _ => None,
    }
}

pub fn parse_kind(err: &CalcError) -> Option<&ParseErrorKind> {
    match err {
        CalcError::Parser(err) => Some(err.kind()),
        _ => None,
    }
}

pub fn resolver_kind(err: &CalcError) -> Option<&ResolverErrorKind> {
    match err {
        CalcError::Resolver(err) => Some(err.kind()),
        _ => None,
    }
}

pub fn eval_kind(err: &CalcError) -> Option<&EvalErrorKind> {
    match err {
        CalcError::Eval(err) => Some(err.kind()),
        _ => None,
    }
}
