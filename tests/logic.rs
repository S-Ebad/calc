//! Comparisons, boolean operators and the conditional.

mod common;

use common::*;

#[test]
fn equality_and_inequality() {
    assert_eq!(solve("5 == 5"), 1.0);
    assert_eq!(solve("5 == 6"), 0.0);
    assert_eq!(solve("5 != 6"), 1.0);
    assert_eq!(solve("5 != 5"), 0.0);
}

#[test]
fn comparison() {
    assert_eq!(solve("3 < 5"), 1.0);
    assert_eq!(solve("5 < 3"), 0.0);
    assert_eq!(solve("5 <= 5"), 1.0);
    assert_eq!(solve("6 > 4"), 1.0);
    assert_eq!(solve("4 > 6"), 0.0);
    assert_eq!(solve("4 >= 4"), 1.0);
}

#[test]
fn boolean_logic() {
    // And logic (both non-zero)
    assert_eq!(solve("true && true"), 1.0);
    assert_eq!(solve("true && false"), 0.0);
    assert_eq!(solve("1 && 5"), 1.0);

    // Or logic (at least one non-zero)
    assert_eq!(solve("true || false"), 1.0);
    assert_eq!(solve("false || false"), 0.0);
    assert_eq!(solve("0 || 3.5"), 1.0);
}

#[test]
fn logical_precedence() {
    // Comparison operators should have higher precedence than logical AND/OR
    assert_eq!(solve("5 > 3 && 2 < 4"), 1.0);
    assert_eq!(solve("5 < 3 || 2 < 4"), 1.0);

    // Arithmetic has higher precedence than comparison
    assert_eq!(solve("2 + 3 == 6 - 1"), 1.0);
}

#[test]
fn basic_ternary() {
    assert_eq!(solve("true ? 10 : 20"), 10.0);
    assert_eq!(solve("false ? 10 : 20"), 20.0);
    assert_eq!(solve("5 > 3 ? 100 : -100"), 100.0);
}

#[test]
fn nested_ternary() {
    // true ? (false ? 1 : 2) : 3 -> 2.0
    assert_eq!(solve("true ? false ? 1 : 2 : 3"), 2.0);
    // false ? 1 : (true ? 2 : 3) -> 2.0
    assert_eq!(solve("false ? 1 : true ? 2 : 3"), 2.0);
}

#[test]
fn complex_expressions_in_branches() {
    assert_eq!(solve("true ? 5 * 4 : 10 / 2"), 20.0);
    assert_eq!(solve("false ? 5 * 4 : 10 / 2"), 5.0);
    assert_eq!(solve("true ? sin(0) : cos(0)"), 0.0);
}

/// A branch that is never taken is still part of the expression, so anything
/// decidable without values has to be rejected either way. Anything that does
/// depend on values must not be.
#[test]
fn an_untaken_branch_is_still_checked_for_arity() {
    let mut session = Session::new();
    session.ok("f(x) = 2x");

    assert!(session.is_err("0 ? f(2, 3) : f(1)"));
    assert!(session.is_err("1 ? f(1) : f(2, 3)"));

    assert!(session.is_err("0 ? sin(1, 2) : sin(1)"));
    assert!(session.is_err("1 ? sin(1) : sin(1, 2)"));
}

#[test]
fn an_untaken_branch_is_not_evaluated() {
    assert_eq!(solve("0 ? 1/0 : 5"), 5.0);
    assert_eq!(solve("1 ? 5 : 1/0"), 5.0);
    assert_eq!(solve("0 ? (-1)! : 5"), 5.0);
}
