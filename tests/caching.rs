//! Memoisation, and the cases that have to invalidate it.

mod common;

use common::*;

#[test]
fn memoization() {
    let mut calc = Calculator::new();
    let _ = calc
        .solve("fib(x) = x <= 1 ? x : fib(x-1) + fib(x-2)")
        .unwrap();

    // build up the cache. If you try to evaluate 100+ without any cache, it'll hit the
    // recursion limit.
    assert_eq!(calc.solve("fib 55").unwrap(), Some(139583862445.0));
    assert_eq!(
        calc.solve("fib 100").unwrap(),
        Some(354224848179262000000.0)
    );
    assert_eq!(
        calc.solve("fib 150").unwrap(),
        Some(9969216677189306000000000000000.0)
    );
    assert_eq!(
        calc.solve("fib 200").unwrap(),
        Some(280571172992510160000000000000000000000000.0)
    );

    // redefine fib to clear the cache
    let _ = calc
        .solve("fib(x) = x <= 1 ? x : fib(x-1) + fib(x-2)")
        .unwrap();

    // attempting to evaluate this will cause a recursion limit error
    assert!(calc.solve("fib 200").is_err());
}

/// The cache is keyed on a name and its arguments, so anything else a result
/// depended on has to invalidate it by hand.

#[test]
fn redefining_a_function_invalidates_its_callers() {
    let mut session = Session::new();
    session.ok("g(a) = a + 1").ok("f(x) = g(x)");

    assert_eq!(session.value("f(1)"), 2.0);

    session.ok("g(a) = a + 100");
    assert_eq!(
        session.value("f(1)"),
        101.0,
        "f cached a result computed from the body that was replaced"
    );
}

#[test]
fn changing_a_global_invalidates_what_read_it() {
    let mut session = Session::new();
    session.ok("k = 1").ok("f(x) = x + k");

    assert_eq!(session.value("f(1)"), 2.0);

    session.ok("k = 10");
    assert_eq!(session.value("f(1)"), 11.0);
}

/// `ans` is written after every answer, so it cannot clear the cache
/// unconditionally without throwing away all memoisation. It still has to
/// invalidate anything that reads it.
#[test]
fn ans_invalidates_only_what_reads_it() {
    let mut session = Session::new();
    session.ok("f(x) = ans + x");

    session.ok("1");
    assert_eq!(session.value("f(1)"), 2.0);

    session.ok("5");
    assert_eq!(session.value("f(1)"), 6.0);
}

#[test]
fn a_cache_survives_answers_it_does_not_depend_on() {
    let mut session = Session::new();
    session.ok("fib(x) = x > 1 ? fib(x-1) + fib(x-2) : x");

    // builds a cache, then sets ans several times over
    assert_eq!(session.value("fib(60)"), 1548008755920.0);
    session.ok("1 + 1").ok("2 + 2");

    // only reachable if the cache survived those answers
    assert_eq!(session.value("fib(79)"), 14472334024676221.0);
}

#[test]
fn an_unrelated_variable_does_not_discard_the_cache() {
    let mut session = Session::new();
    session.ok("fib(x) = x > 1 ? fib(x-1) + fib(x-2) : x");

    assert_eq!(session.value("fib(60)"), 1548008755920.0);
    session.ok("unrelated = 1");

    assert_eq!(session.value("fib(79)"), 14472334024676221.0);
}
