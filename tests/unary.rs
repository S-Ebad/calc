//! Prefix signs and the postfix factorial.

mod common;

use common::*;

#[test]
fn large_factorial_overflow() {
    // The implementation caps factorial at 170
    assert!(solve_is_err("171!"));
}

#[test]
fn unary_neg() {
    assert_eq!(solve("-5"), -5.0);
    assert_eq!(solve("-0"), 0.0);
    assert_eq!(solve("-100"), -100.0);
    assert_eq!(solve("--5"), 5.0);
}
#[test]
fn unary_neg_in_expr() {
    assert_eq!(solve("10 + -3"), 7.0);
    assert_eq!(solve("10 - -3"), 13.0);
    assert_eq!(solve("5 * -2"), -10.0);
}
#[test]
fn unary_pos() {
    assert_eq!(solve("+5"), 5.0);
    assert_eq!(solve("+0"), 0.0);
    assert_eq!(solve("+100"), 100.0);
}

#[test]
fn unary_neg_with_functions() {
    assert_eq!(solve("abs -5"), 5.0);
    assert_eq!(solve("abs -100"), 100.0);
    assert_eq!(solve("floor -3.2"), -4.0);
    assert_eq!(solve("ceil -3.9"), -3.0);
}

#[test]
fn double_negation() {
    assert_eq!(solve("--5"), 5.0);
    assert_eq!(solve("--0"), 0.0);
    assert_eq!(solve("1 + --3"), 4.0);
}

#[test]
fn factorial_5() {
    assert_eq!(solve("5!"), 120.0);
    assert_eq!(solve("3!"), 6.0);
    assert_eq!(solve("6!"), 720.0);
    assert_eq!(solve("10!"), 3628800.0);
}
#[test]
fn factorial_0() {
    assert_eq!(solve("0!"), 1.0);
    assert_eq!(solve("1!"), 1.0);
    assert_eq!(solve("2!"), 2.0);
}

#[test]
fn factorial_negative_is_error() {
    assert!(solve_is_err("(-1)!"));
    assert!(solve_is_err("(-5)!"));
}

#[test]
fn factorial_non_integer_is_error() {
    assert!(solve_is_err("2.5!"));
    assert!(solve_is_err("0.1!"));
}

#[test]
fn factorial_in_expr() {
    assert_eq!(solve("5! + 1"), 121.0);
    assert_eq!(solve("2 * 3!"), 12.0);
    assert_eq!(solve("3! ^ 2"), 36.0);
    assert_eq!(solve("(2 + 1)!"), 6.0);
}
