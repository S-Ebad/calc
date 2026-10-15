//! Built-in constants and implicit multiplication.

mod common;

use common::*;

#[test]
fn constants() {
    assert_eq!(solve("pi"), round(std::f64::consts::PI));
    assert_eq!(solve("e"), round(std::f64::consts::E));
    assert_eq!(solve("inf"), f64::INFINITY);
    assert_eq!(solve("-inf"), f64::NEG_INFINITY);
}

#[test]
fn boolean_constants() {
    assert_eq!(solve("true"), 1.0);
    assert_eq!(solve("false"), 0.0);
}

#[test]
fn implicit_mul() {
    assert_eq!(solve("2(3 + 1)"), 8.0);
    assert_eq!(solve("(2 + 3)4"), 20.0);
    assert_eq!(solve("2pi"), round(2.0 * std::f64::consts::PI));
    assert_eq!(solve("3(2 + 2)"), 12.0);
    assert_eq!(solve("2e"), round(2.0 * std::f64::consts::E));
    assert_eq!(solve("2true"), 2.0);
    assert_eq!(solve("5false"), 0.0);
}

#[test]
fn scientific_notation_vs_euler() {
    assert_eq!(solve("9e"), round(9.0 * std::f64::consts::E));
    assert_eq!(solve("9e2"), 900.0);
    assert!(solve_is_err("9e9e9"));
}

#[test]
fn implicit_function_call() {
    assert_eq!(solve("sin pi"), 0.0);
    assert_eq!(solve("cos pi"), -1.0);
    assert_eq!(solve("abs -5"), 5.0);
}

#[test]
fn implicit_chain() {
    assert_eq!(solve("pi sin pi"), 0.0);
    assert_eq!(solve("2 cos pi"), -2.0);
}

#[test]
fn constants_in_expressions() {
    assert_eq!(solve("pi * 2"), round(2.0 * std::f64::consts::PI));
    assert_eq!(solve("e ^ 2"), round(std::f64::consts::E.powi(2)));
    assert_eq!(solve("pi / pi"), 1.0);
    assert_eq!(solve("e / e"), 1.0);
}

#[test]
fn inf_arithmetic() {
    assert_eq!(solve("inf + 1"), f64::INFINITY);
    assert_eq!(solve("inf * 2"), f64::INFINITY);
    assert_eq!(solve("-inf - 1"), f64::NEG_INFINITY);
    assert!(solve_is_err("inf - inf"));
}

#[test]
fn tau_and_phi() {
    approx(solve("tau"), std::f64::consts::TAU);
    approx(solve("tau / 2"), std::f64::consts::PI);

    approx(solve("phi"), 1.618_033_988_749_895);
    // phi squared is phi plus one
    approx(solve("phi^2 - phi"), 1.0);
}
