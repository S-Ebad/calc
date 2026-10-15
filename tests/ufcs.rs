//! Dot-call syntax, and the lexing it has to stay clear of.

mod common;

use common::*;

#[test]
fn no_arg_dot() {
    assert_eq!(solve("10.max(5)"), 10.0f64.max(5.0));
    assert_eq!(solve("10.max(20)"), 20.0);
    assert_eq!(solve("3.max(1)"), 3.0);
}

#[test]
fn sqrt() {
    assert_eq!(solve("9.sqrt()"), 9.0f64.sqrt());
    assert_eq!(solve("16.sqrt()"), 4.0);
    assert_eq!(solve("0.sqrt()"), 0.0);
}

#[test]
fn chained() {
    assert_eq!(solve("10.max(5).min(3)"), 10.0f64.max(5.0).min(3.0));
    assert_eq!(solve("10.max(5).min(3)"), 3.0);
    assert_eq!(solve("1.max(5).min(3)"), 3.0);
}

#[test]
fn float_literal() {
    assert_eq!(solve("10.5"), 10.5);
    assert_eq!(solve("0.5"), 0.5);
    assert_eq!(solve("123.456"), 123.456);
}

#[test]
fn float_then_call() {
    assert_eq!(solve("10.5.max(5)"), 10.5f64.max(5.0));
    assert_eq!(solve("10.5.max(20)"), 20.0);
    assert_eq!(solve("1.5.min(0.5)"), 0.5);
}

#[test]
fn plain_arithmetic() {
    assert_eq!(solve("10 + 5"), 15.0);
    assert_eq!(solve("10 - 5"), 5.0);
    assert_eq!(solve("10 * 5"), 50.0);
}

#[test]
fn exponent() {
    assert_eq!(solve("9e9"), 9e9);
    assert_eq!(solve("1e3"), 1e3);
    assert_eq!(solve("2e0"), 2.0);
}

#[test]
fn exponent_signed() {
    assert_eq!(solve("9e-9"), 9e-9);
    assert_eq!(solve("9e+9"), 9e9);
    assert_eq!(solve("1e-3"), 1e-3);
}

#[test]
fn exponent_then_call() {
    assert_eq!(solve("9e2.max(5)"), 9e2f64.max(5.0));
    assert_eq!(solve("9e2.max(1000)"), 1000.0);
    assert_eq!(solve("1e1.min(5)"), 5.0);
}

#[test]
fn double_exponent_err() {
    assert!(solve_is_err("9e9e9"));
    assert!(solve_is_err("1e1e1"));
    assert!(solve_is_err("2e2e2"));
}

#[test]
fn exponent_bad_decimal_err() {
    assert!(solve_is_err("9e9.2"));
    assert!(solve_is_err("1e1.5"));
    assert!(solve_is_err("2e2.9"));
}

#[test]
fn trailing_dot_err() {
    assert!(solve_is_err("10."));
    assert!(solve_is_err("5."));
    assert!(solve_is_err("100."));
}

#[test]
fn dot_bad_token_err() {
    assert!(solve_is_err("10. + 5"));
    assert!(solve_is_err("5. + 1"));
    assert!(solve_is_err("3. * 2"));
}

#[test]
fn user_fn() {
    let mut calc = Calculator::new();
    calc.solve("f(x) = 2x").unwrap();
    assert_eq!(calc.solve("5.f()").unwrap(), Some(10.0));
    assert_eq!(calc.solve("3.f()").unwrap(), Some(6.0));
    assert_eq!(calc.solve("0.f()").unwrap(), Some(0.0));
}

#[test]
fn user_fn_extra_args() {
    let mut calc = Calculator::new();
    calc.solve("g(x, y) = x + y").unwrap();
    assert_eq!(calc.solve("5.g(3)").unwrap(), Some(8.0));
    assert_eq!(calc.solve("10.g(5)").unwrap(), Some(15.0));
    assert_eq!(calc.solve("0.g(0)").unwrap(), Some(0.0));
}

#[test]
fn user_fn_chained_builtin() {
    let mut calc = Calculator::new();
    calc.solve("f(x) = 2x").unwrap();
    assert_eq!(calc.solve("5.f().max(3)").unwrap(), Some(10.0));
    assert_eq!(calc.solve("1.f().max(3)").unwrap(), Some(3.0));
    assert_eq!(calc.solve("5.f().min(3)").unwrap(), Some(3.0));
}

#[test]
fn exponent_user_fn() {
    let mut calc = Calculator::new();
    calc.solve("f(x) = x + 1").unwrap();
    assert_eq!(calc.solve("9e1.f()").unwrap(), Some(91.0));
    assert_eq!(calc.solve("1e2.f()").unwrap(), Some(101.0));
    assert_eq!(calc.solve("1e0.f()").unwrap(), Some(2.0));
}

#[test]
fn user_fn_chained() {
    let mut calc = Calculator::new();
    calc.solve("f(x) = 2x").unwrap();
    calc.solve("g(x) = x + 1").unwrap();
    assert_eq!(calc.solve("5.f().g()").unwrap(), Some(11.0));
    assert_eq!(calc.solve("0.f().g()").unwrap(), Some(1.0));
    assert_eq!(calc.solve("3.g().f()").unwrap(), Some(8.0));
}
