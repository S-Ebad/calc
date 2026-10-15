//! What a name in a function body refers to.
//!
//! The rule is deliberately two levels: a name is the function's own
//! parameter if it has one, otherwise the global of that name, looked up when
//! the call happens. A body does not see its caller's parameters.

mod common;

use common::*;

#[test]
fn a_parameter_shadows_a_global_of_the_same_name() {
    let mut session = Session::new();
    session.ok("x = 10").ok("f(x) = x");

    assert_eq!(session.value("f(1)"), 1.0);
    assert_eq!(session.value("x"), 10.0, "the global is untouched");
}

#[test]
fn a_body_reads_globals() {
    let mut session = Session::new();
    session.ok("k = 3").ok("f() = k");

    assert_eq!(session.value("f()"), 3.0);
}

/// The point of resolving a body at call time: a name it reads need not exist
/// when the function is written.
#[test]
fn a_global_may_be_defined_after_the_function_that_reads_it() {
    let mut session = Session::new();
    session.ok("f(x) = x + later");

    assert!(session.is_err("f(1)"));

    session.ok("later = 10");
    assert_eq!(session.value("f(1)"), 11.0);
}

#[test]
fn changing_a_global_changes_what_the_function_returns() {
    let mut session = Session::new();
    session.ok("k = 2").ok("f(x) = k * x");

    assert_eq!(session.value("f(10)"), 20.0);

    session.ok("k = 5");
    assert_eq!(session.value("f(10)"), 50.0);
}

#[test]
fn a_body_cannot_see_its_callers_parameters() {
    let mut session = Session::new();
    session.ok("callee(a) = a + x").ok("caller(x) = callee(1)");

    assert!(matches!(
        eval_kind(&session.err("caller(7)")),
        Some(EvalErrorKind::Resolution(_))
    ));
}

#[test]
fn nor_those_of_a_caller_further_up() {
    let mut session = Session::new();
    session
        .ok("c(n) = n + w")
        .ok("b(m) = c(m)")
        .ok("a(w) = b(1)");

    assert!(session.is_err("a(3)"));
}

/// A caller's parameter must not hide a global the callee meant to read,
/// which is what made the old behaviour hard to reason about.
#[test]
fn a_callers_parameter_does_not_capture_a_global() {
    let mut session = Session::new();
    session
        .ok("x = 10")
        .ok("callee(a) = a + x")
        .ok("caller(x) = callee(1)");

    assert_eq!(session.value("callee(1)"), 11.0);

    // the caller binding x must not change what callee reads
    assert_eq!(session.value("caller(7)"), 11.0);
    assert_eq!(session.value("caller(99)"), 11.0);
}

#[test]
fn parameters_keep_their_order() {
    let mut session = Session::new();
    session.ok("f(a, b, c) = a * 100 + b * 10 + c");

    assert_eq!(session.value("f(1, 2, 3)"), 123.0);
    assert_eq!(session.value("f(3, 2, 1)"), 321.0);
}

#[test]
fn a_parameter_may_be_read_many_times() {
    let mut session = Session::new();
    session.ok("f(x) = x + x + x + x");

    assert_eq!(session.value("f(2)"), 8.0);
}

#[test]
fn a_parameter_still_multiplies_implicitly() {
    let mut session = Session::new();
    session.ok("f(x) = x(2)");

    assert_eq!(session.value("f(5)"), 10.0);
}

#[test]
fn a_parameter_works_as_a_dot_call_receiver() {
    let mut session = Session::new();
    session.ok("f(x) = x.max(3)");

    assert_eq!(session.value("f(1)"), 3.0);
    assert_eq!(session.value("f(9)"), 9.0);
}

#[test]
fn recursion_binds_the_parameter_at_each_level() {
    let mut session = Session::new();
    session.ok("countdown(n) = n > 0 ? n + countdown(n - 1) : 0");

    assert_eq!(session.value("countdown(4)"), 10.0);
}
