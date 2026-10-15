//! Arithmetic, precedence and numeric precision.

mod common;

use common::*;

#[test]
fn addition() {
    assert_eq!(solve("1 + 2"), 3.0);
    assert_eq!(solve("0 + 0"), 0.0);
    assert_eq!(solve("-3 + 3"), 0.0);
    assert_eq!(solve("100 + 200"), 300.0);
}
#[test]
fn subtraction() {
    assert_eq!(solve("10 - 4"), 6.0);
    assert_eq!(solve("0 - 0"), 0.0);
    assert_eq!(solve("5 - 10"), -5.0);
    assert_eq!(solve("100 - 1"), 99.0);
}
#[test]
fn multiplication() {
    assert_eq!(solve("3 * 4"), 12.0);
    assert_eq!(solve("0 * 999"), 0.0);
    assert_eq!(solve("-2 * 5"), -10.0);
    assert_eq!(solve("-3 * -3"), 9.0);
}
#[test]
fn division() {
    assert_eq!(solve("10 / 4"), 2.5);
    assert_eq!(solve("0 / 5"), 0.0);
    assert_eq!(solve("9 / 3"), 3.0);
    assert_eq!(solve("-8 / 2"), -4.0);
}
#[test]
fn modulo() {
    assert_eq!(solve("10 % 3"), 1.0);
    assert_eq!(solve("9 % 3"), 0.0);
    assert_eq!(solve("1 % 5"), 1.0);
    assert_eq!(solve("0 % 7"), 0.0);
}
#[test]
fn exponentiation() {
    assert_eq!(solve("2 ^ 10"), 1024.0);
    assert_eq!(solve("5 ^ 0"), 1.0);
    assert_eq!(solve("5 ^ 1"), 5.0);
    assert_eq!(solve("3 ^ 3"), 27.0);
    assert_eq!(solve("0 ^ 0"), 1.0);
}

#[test]
fn precedence() {
    assert_eq!(solve("2 + 3 * 4"), 14.0);
    assert_eq!(solve("(2 + 3) * 4"), 20.0);
    assert_eq!(solve("10 - 2 * 3"), 4.0);
    assert_eq!(solve("10 / 2 + 3"), 8.0);
}

#[test]
fn precision() {
    // 0.1 + 0.2 is a classic floating-point pitfall; the formatter handles it
    assert_eq!(solve("0.1 + 0.2"), 0.3);
    assert_eq!(solve("0.3 - 0.1"), 0.2);
    assert_eq!(solve("0.1 * 3"), 0.3);
}

#[test]
fn nested_parens() {
    assert_eq!(solve("((2 + 3) * (1 + 1))"), 10.0);
    assert_eq!(solve("((10 - 5) * (2 + 3))"), 25.0);
    assert_eq!(solve("(((2)))"), 2.0);
}

#[test]
fn right_assoc_power() {
    // 2^3^2 = 2^(3^2) = 2^9 = 512  (right-associative)
    assert_eq!(solve("2^3^2"), 512.0);
    assert_eq!(solve("2^2^3"), 256.0); // 2^(2^3) = 2^8
    assert_eq!(solve("3^2^2"), 81.0); // 3^(2^2) = 3^4
}

#[test]
fn large_numbers() {
    assert_eq!(solve("1000000 * 1000000"), 1_000_000_000_000.0);
    assert_eq!(solve("999999 + 1"), 1_000_000.0);
    assert_eq!(solve("10 ^ 12"), 1_000_000_000_000.0);
}

#[test]
fn negative_results() {
    assert_eq!(solve("3 - 10"), -7.0);
    assert_eq!(solve("-5 - 5"), -10.0);
    assert_eq!(solve("-5 * -5 - 30"), -5.0);
}

#[test]
fn chained_operations() {
    assert_eq!(solve("1 + 2 + 3 + 4 + 5"), 15.0);
    assert_eq!(solve("100 - 10 - 10 - 10"), 70.0);
    assert_eq!(solve("2 * 3 * 4"), 24.0);
    assert_eq!(solve("100 / 2 / 5"), 10.0);
}

#[test]
fn mixed_operations() {
    assert_eq!(solve("2 + 3 * 4 - 1"), 13.0);
    assert_eq!(solve("10 / 2 * 3"), 15.0);
    assert_eq!(solve("2 ^ 3 + 1"), 9.0);
    assert_eq!(solve("(2 + 3) ^ 2"), 25.0);
}

#[test]
fn floating_point_precision_chain() {
    // Repeated addition of 0.1 to check rounding stability
    assert_eq!(
        solve("0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1 + 0.1"),
        1.0
    );
}

#[test]
fn power_of_negatives() {
    // Odd root of a negative number via sqrt(x, n)
    assert_eq!(solve("sqrt(-8, 3)"), -2.0);
    // Even root of negative should error
    assert!(solve_is_err("sqrt(-4, 2)"));
}

#[test]
fn modulo_with_decimals() {
    // 10.5 % 3 = 1.5
    assert_eq!(solve("10.5 % 3"), 1.5);
}
