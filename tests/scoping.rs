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

/// A parameter shadows a function of the same name too. The parser has to know
/// this while it reads the body, because a call and a value build different
/// trees: read as the function, this body would be 'f(-2) * 3'.
#[test]
fn a_parameter_shadows_a_function_of_the_same_name() {
    let mut session = Session::new();
    session.ok("f(x) = x").ok("g(f) = f - 2*3");

    assert_eq!(session.value("g(10)"), 4.0);
    assert_eq!(session.value("f(10)"), 10.0, "the function is untouched");
}

#[test]
fn a_shadowing_parameter_reads_as_a_value_wherever_it_appears() {
    let mut session = Session::new();
    session.ok("f(x) = x");

    // at the end of the body, before an argument, and as a factor
    session.ok("a(f) = 2f").ok("b(f, y) = f y").ok("c(f) = f(2)");

    assert_eq!(session.value("a(5)"), 10.0);
    assert_eq!(session.value("b(3, 4)"), 12.0);
    assert_eq!(session.value("c(5)"), 10.0, "applied, it multiplies");
}

#[test]
fn a_parameter_shadows_a_built_in() {
    let mut session = Session::new();
    session.ok("g(sin) = sin + 1");

    assert_eq!(session.value("g(2)"), 3.0);
    assert_eq!(session.value("sin(0)"), 0.0, "the built-in is untouched");
}

/// Shadowing reaches the body only: the name being defined is the one thing on
/// the line its parameters do not cover.
#[test]
fn a_function_may_name_a_parameter_after_itself() {
    let mut session = Session::new();
    session.ok("f(x) = x").ok("f(f) = f + 1");

    assert_eq!(session.value("f(2)"), 3.0);
}

/// Lines that are not definitions parse exactly as before: the function is
/// still a function.
#[test]
fn outside_a_definition_a_function_is_still_called() {
    let mut session = Session::new();
    session.ok("f(x) = 2x");

    assert_eq!(session.value("f -2*3"), -12.0, "an implicit call, then the product");
    assert!(session.is_err("max(1, f)"));
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
