//! Inputs that have to be rejected.

mod common;

use common::*;

#[test]
fn division_by_zero() {
    assert!(solve_is_err("1 / 0"));
    assert!(solve_is_err("100 / 0"));
    assert!(solve_is_err("-5 / 0"));
}

#[test]
fn power_zero_to_negative() {
    assert!(solve_is_err("0 ^ -1"));
    assert!(solve_is_err("0 ^ -2"));
    assert!(solve_is_err("pow(0, -1)"));
    assert!(solve_is_err("pow(0, -5)"));
}

#[test]
fn recip_zero() {
    assert!(solve_is_err("recip(0)"));
}

#[test]
fn mismatched_open() {
    assert!(solve_is_err("(1 + 2"));
    assert!(solve_is_err("((3 + 4)"));
}

#[test]
fn mismatched_close() {
    assert!(solve_is_err("1 + 2)"));
    assert!(solve_is_err("(1 + 2))"));
}

#[test]
fn empty_expr() {
    assert!(solve_is_err(""));
}

#[test]
fn invalid_token() {
    assert!(solve_is_err("1 @ 2"));
    assert!(solve_is_err("1 ~ 2"));
    assert!(solve_is_err("1 $ 2"));
}

#[test]
fn missing_operator() {
    assert!(solve_is_err("1 2"));
    assert!(solve_is_err("3 4 5"));
}

#[test]
fn tan_asymptote() {
    assert!(solve_is_err("tan(pi / 2)"));
    assert!(solve_is_err("tan(3 * pi / 2)"));
}

#[test]
fn sqrt_negative() {
    assert!(solve_is_err("sqrt(-1)"));
}

#[test]
fn ln_non_positive() {
    assert!(solve_is_err("ln(-1)"));
}

#[test]
fn log_non_positive() {
    assert_eq!(solve("log(0)"), f64::NEG_INFINITY);
    assert!(solve_is_err("log(-1)"));
}

#[test]
fn deeply_nested_parens() {
    // valid deep nesting shouldn't error
    assert_eq!(solve("((((((1 + 2))))))"), 3.0);
}

#[test]
fn whitespace_handling() {
    assert_eq!(solve("1+2"), 3.0);
    assert_eq!(solve("1   +   2"), 3.0);
    assert_eq!(solve("  10  /  2  "), 5.0);
}

#[test]
fn invalid_function_application() {
    let mut calc = Calculator::new();
    calc.solve("x = 5").unwrap();
    // Cannot multiply a variable by a list of arguments as if it were a function
    assert!(calc.solve("x(1, 2)").is_err());
}

#[test]
fn invalid_function() {
    let mut calc = Calculator::new();

    // Func not yet defined
    assert!(calc.solve("5 + Func(5)").is_err())
}
