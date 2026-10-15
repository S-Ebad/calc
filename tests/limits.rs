//! Inputs that used to exhaust the native stack.
//!
//! Resolving and evaluating both recurse, and so does freeing the tree, so a
//! deep enough expression could abort the process instead of reporting
//! anything. These assert that each shape is reported rather than fatal; a
//! regression here does not fail the assertion, it kills the test runner.

mod common;

use common::*;

#[test]
fn a_long_flat_chain_is_reported() {
    let expr = format!("{}1", "1+".repeat(5_000));

    assert!(solve_is_err(&expr));
}

#[test]
fn deeply_nested_parentheses_are_reported() {
    let expr = format!("{}1{}", "(".repeat(5_000), ")".repeat(5_000));

    // the parser refuses it before anything deeper runs
    assert!(matches!(
        parse_kind(&err(&expr)),
        Some(ParseErrorKind::ExpressionTooDeep)
    ));
}

/// Nesting and call depth multiply: a body well within reach on its own is
/// not within reach once every caller's frames are underneath it.
#[test]
fn a_body_within_reach_is_still_bounded_at_depth() {
    let mut session = Session::new();
    session.ok(&format!("f(x) = x > 0 ? f(x - 1){} : 0", "+1".repeat(40)));

    assert_eq!(session.value("f(2)"), 80.0);

    // the body parsed on its own, so this is only reachable once the frames
    // of every call beneath it are counted as well
    assert!(matches!(
        eval_kind(&session.err("f(70)")),
        Some(EvalErrorKind::Resolution(_))
    ));
}

#[test]
fn nesting_within_reach_still_evaluates() {
    assert_eq!(solve("((((((1 + 2))))))"), 3.0);
    assert_eq!(solve(&format!("{}1", "1+".repeat(30))), 31.0);
    assert_eq!(
        solve(&format!("{}1{}", "(".repeat(30), ")".repeat(30))),
        1.0
    );
}

/// Unary plus builds nothing, so a run of them is not nesting and must not be
/// counted as such.
#[test]
fn a_long_run_of_unary_plus_is_not_nesting() {
    let expr = format!("{}1", "+".repeat(10_000));

    assert_eq!(solve(&expr), 1.0);
}

/// Negation does build a node, so a long chain of it is real depth.
#[test]
fn a_long_run_of_negation_is_nesting() {
    assert!(solve_is_err(&format!("{}1", "-".repeat(5_000))));
}

#[test]
fn runaway_recursion_reports_the_recursion_limit() {
    let mut session = Session::new();
    session.ok("r(x) = r(x)");

    assert!(matches!(
        eval_kind(&session.err("r(1)")),
        Some(EvalErrorKind::RecursionLimit)
    ));
}

#[test]
fn recursion_within_the_limit_is_fine() {
    let mut session = Session::new();
    session.ok("down(n) = n > 0 ? down(n - 1) : 0");

    assert_eq!(session.value("down(50)"), 0.0);
}
