//! Assignment, reassignment and `ans`.

mod common;

use common::*;

#[test]
fn assignment() {
    let mut calc = Calculator::new();
    calc.solve("x = 5").unwrap();
    assert_eq!(calc.solve("x + 3").unwrap().unwrap(), 8.0);
    assert_eq!(calc.solve("x * 2").unwrap().unwrap(), 10.0);
    assert_eq!(calc.solve("x ^ 2").unwrap().unwrap(), 25.0);
}

#[test]
fn multiple_assignments() {
    let mut calc = Calculator::new();
    calc.solve("a = 3").unwrap();
    calc.solve("b = 4").unwrap();
    assert_eq!(calc.solve("a + b").unwrap().unwrap(), 7.0);
    assert_eq!(calc.solve("a * b").unwrap().unwrap(), 12.0);
}

#[test]
fn reassignment() {
    let mut calc = Calculator::new();
    calc.solve("x = 5").unwrap();
    calc.solve("x = 10").unwrap();
    assert_eq!(calc.solve("x").unwrap().unwrap(), 10.0);
}

#[test]
fn ans_updates() {
    let mut calc = Calculator::new();
    calc.solve("10 + 5").unwrap();
    assert_eq!(calc.solve("ans").unwrap().unwrap(), 15.0);
    calc.solve("3 * 3").unwrap();
    assert_eq!(calc.solve("ans").unwrap().unwrap(), 9.0);
}

#[test]
fn ans_chaining() {
    let mut calc = Calculator::new();
    calc.solve("10").unwrap();
    assert_eq!(calc.solve("ans * 2").unwrap().unwrap(), 20.0);
    assert_eq!(calc.solve("ans + 5").unwrap().unwrap(), 25.0);
}

#[test]
fn ans_uninitialized_error() {
    assert!(solve_is_err("ans"));
}

#[test]
fn unknown_variable_error() {
    assert!(solve_is_err("foo"));
    assert!(solve_is_err("bar"));
    assert!(solve_is_err("xyz"));
}

#[test]
fn variable_in_complex_expr() {
    let mut calc = Calculator::new();
    calc.solve("x = 3").unwrap();
    calc.solve("y = 4").unwrap();
    assert_eq!(round(calc.solve("sqrt(x^2 + y^2)").unwrap().unwrap()), 5.0); // pythagorean
    assert_eq!(calc.solve("2x + 3y").unwrap().unwrap(), 18.0); // implicit mul with vars
    assert_eq!(calc.solve("(x + y) ^ 2").unwrap().unwrap(), 49.0);
}

#[test]
fn variable_reassign_affects_expr() {
    let mut calc = Calculator::new();
    calc.solve("x = 2").unwrap();
    assert_eq!(calc.solve("x ^ 3").unwrap().unwrap(), 8.0);
    calc.solve("x = 3").unwrap();
    assert_eq!(calc.solve("x ^ 3").unwrap().unwrap(), 27.0);
}

#[test]
fn ans_persists_across_errors() {
    let mut calc = Calculator::new();
    calc.solve("10").unwrap();
    let _ = calc.solve("1 / 0"); // error shouldn't wipe ans
    assert_eq!(calc.solve("ans").unwrap().unwrap(), 10.0);
}
