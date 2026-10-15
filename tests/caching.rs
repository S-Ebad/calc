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
