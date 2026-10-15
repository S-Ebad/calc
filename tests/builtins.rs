//! The built-in function library.

mod common;

use common::*;

// trig
#[test]
fn sin() {
    assert_eq!(solve("sin(0)"), 0.0);
    assert_eq!(solve("sin(pi)"), 0.0);
    assert_eq!(solve("sin(pi / 2)"), 1.0);
    assert_eq!(solve("sin(pi / 6)"), round(0.5));
}
#[test]
fn cos() {
    assert_eq!(solve("cos(0)"), 1.0);
    assert_eq!(solve("cos(pi)"), -1.0);
    assert_eq!(solve("cos(pi / 2)"), 0.0);
    assert_eq!(solve("cos(pi / 3)"), round(0.5));
}
#[test]
fn tan() {
    assert_eq!(solve("tan(0)"), 0.0);
    assert_eq!(solve("tan(pi / 4)"), 1.0);
}

// sqrt variants
#[test]
fn sqrt() {
    assert_eq!(solve("sqrt(9)"), 3.0);
    assert_eq!(solve("sqrt(4)"), 2.0);
    assert_eq!(solve("sqrt(1)"), 1.0);
    assert_eq!(solve("sqrt(0)"), 0.0);
}
#[test]
fn nth_root() {
    assert_eq!(solve("sqrt(27, 3)"), 3.0);
    assert_eq!(solve("sqrt(16, 4)"), 2.0);
    assert_eq!(solve("sqrt(32, 5)"), 2.0);
}

#[test]
fn atan2() {
    assert_eq!(solve("atan2(1, 1)"), round(std::f64::consts::FRAC_PI_4));
    assert_eq!(
        solve("atan2(1, -1)"),
        round(3.0 * std::f64::consts::FRAC_PI_4)
    );
    assert_eq!(
        solve("atan2(-1, -1)"),
        round(-3.0 * std::f64::consts::FRAC_PI_4)
    );
    assert_eq!(solve("atan2(-1, 1)"), round(-std::f64::consts::FRAC_PI_4));

    assert_eq!(solve("atan2(0, 1)"), 0.0);
    assert_eq!(solve("atan2(1, 0)"), round(std::f64::consts::FRAC_PI_2));
    assert_eq!(solve("atan2(0, -1)"), round(std::f64::consts::PI));
    assert_eq!(solve("atan2(-1, 0)"), round(-std::f64::consts::FRAC_PI_2));

    assert_eq!(solve("atan2(1, 1)"), round(solve("atan(1 / 1)")));
    assert_eq!(solve("atan2(3, 4)"), round(solve("atan(3 / 4)")));

    assert!(solve_is_err("atan2(1)"));
    assert!(solve_is_err("atan2(1, 2, 3)"));
}

// basic math
#[test]
fn abs() {
    assert_eq!(solve("abs(-7)"), 7.0);
    assert_eq!(solve("abs(7)"), 7.0);
    assert_eq!(solve("abs(0)"), 0.0);
    assert_eq!(solve("abs(-100)"), 100.0);
}
#[test]
fn floor() {
    assert_eq!(solve("floor(3.9)"), 3.0);
    assert_eq!(solve("floor(3.0)"), 3.0);
    assert_eq!(solve("floor(-3.1)"), -4.0);
    assert_eq!(solve("floor(0.9)"), 0.0);
}
#[test]
fn ceil() {
    assert_eq!(solve("ceil(3.1)"), 4.0);
    assert_eq!(solve("ceil(3.0)"), 3.0);
    assert_eq!(solve("ceil(-3.9)"), -3.0);
    assert_eq!(solve("ceil(0.1)"), 1.0);
}
#[test]
fn round_() {
    assert_eq!(solve("round(3.5)"), 4.0);
    assert_eq!(solve("round(3.4)"), 3.0);
    assert_eq!(solve("round(-3.5)"), -4.0);
    assert_eq!(solve("round(0.5)"), 1.0);
}

// int/trunc
#[test]
fn int() {
    assert_eq!(solve("int(3.9)"), 3.0);
    assert_eq!(solve("int(-2.5)"), -2.0);
    assert_eq!(solve("int(0.9)"), 0.0);
    assert_eq!(solve("int(5.0)"), 5.0);
}
#[test]
fn trunc() {
    assert_eq!(solve("trunc(3.9)"), 3.0);
    assert_eq!(solve("trunc(-2.5)"), -2.0);
    assert_eq!(solve("trunc(0.1)"), 0.0);
    assert_eq!(solve("trunc(-0.9)"), 0.0);
}

// logs / exp
#[test]
fn ln() {
    assert_eq!(solve("ln(e)"), 1.0);
    assert_eq!(solve("ln(1)"), 0.0);
    assert_eq!(solve("ln(e ^ 2)"), 2.0);
}
#[test]
fn log10() {
    assert_eq!(solve("log(100)"), 2.0);
    assert_eq!(solve("log(1)"), 0.0);
    assert_eq!(solve("log(10)"), 1.0);
    assert_eq!(solve("log(1000)"), 3.0);
}
#[test]
fn log_base() {
    assert_eq!(solve("log(8, 2)"), 3.0);
    assert_eq!(solve("log(27, 3)"), 3.0);
    assert_eq!(solve("log(1, 5)"), 0.0);
}
#[test]
fn exp() {
    assert_eq!(solve("exp(1)"), round(std::f64::consts::E));
    assert_eq!(solve("exp(0)"), 1.0);
    assert_eq!(solve("exp(2)"), round(std::f64::consts::E.powi(2)));
}

// power / roots
#[test]
fn pow() {
    assert_eq!(solve("pow(3, 4)"), 81.0);
    assert_eq!(solve("pow(2, 0)"), 1.0);
    assert_eq!(solve("pow(5, 1)"), 5.0);
    assert_eq!(solve("pow(2, 8)"), 256.0);
    assert_eq!(solve("pow(0, 0)"), 1.0);
}
#[test]
fn cbrt() {
    assert_eq!(solve("cbrt(27)"), 3.0);
    assert_eq!(solve("cbrt(8)"), 2.0);
    assert_eq!(solve("cbrt(1)"), 1.0);
    assert_eq!(solve("cbrt(0)"), 0.0);
}

// misc math
#[test]
fn recip() {
    assert_eq!(solve("recip(4)"), 0.25);
    assert_eq!(solve("recip(1)"), 1.0);
    assert_eq!(solve("recip(2)"), 0.5);
    assert_eq!(solve("recip(-1)"), -1.0);
}
#[test]
fn max() {
    assert_eq!(solve("max(1, 5, 3)"), 5.0);
    assert_eq!(solve("max(0, 0, 0)"), 0.0);
    assert_eq!(solve("max(-3, -1, -2)"), -1.0);
    assert_eq!(solve("max(1, 1)"), 1.0);
}
#[test]
fn min() {
    assert_eq!(solve("min(4, 2, 9)"), 2.0);
    assert_eq!(solve("min(0, 0, 0)"), 0.0);
    assert_eq!(solve("min(-3, -1, -2)"), -3.0);
    assert_eq!(solve("min(1, 1)"), 1.0);
}

// angle conversion
#[test]
fn deg() {
    assert_eq!(solve("deg(pi)"), 180.0);
    assert_eq!(solve("deg(0)"), 0.0);
    assert_eq!(solve("deg(2pi)"), 360.0);
    assert_eq!(solve("deg(pi / 2)"), 90.0);
}
#[test]
fn rad() {
    assert_eq!(solve("rad(180)"), round(std::f64::consts::PI));
    assert_eq!(solve("rad(0)"), 0.0);
    assert_eq!(solve("rad(360)"), round(2.0 * std::f64::consts::PI));
    assert_eq!(solve("rad(90)"), round(std::f64::consts::FRAC_PI_2));
}

// trig inverse
#[test]
fn asin() {
    assert_eq!(solve("asin(1)"), round(std::f64::consts::FRAC_PI_2));
    assert_eq!(solve("asin(0)"), 0.0);
    assert_eq!(solve("asin(-1)"), round(-std::f64::consts::FRAC_PI_2));
}
#[test]
fn acos() {
    assert_eq!(solve("acos(1)"), 0.0);
    assert_eq!(solve("acos(0)"), round(std::f64::consts::FRAC_PI_2));
    assert_eq!(solve("acos(-1)"), round(std::f64::consts::PI));
}
#[test]
fn atan() {
    assert_eq!(solve("atan(1)"), round(std::f64::consts::FRAC_PI_4));
    assert_eq!(solve("atan(0)"), 0.0);
    assert_eq!(solve("atan(-1)"), round(-std::f64::consts::FRAC_PI_4));
}

// hyperbolic
#[test]
fn sinh() {
    assert_eq!(solve("sinh(0)"), 0.0);
    assert_eq!(solve("sinh(1)"), round(1_f64.sinh()));
    assert_eq!(solve("sinh(-1)"), round(-1_f64.sinh()));
}
#[test]
fn cosh() {
    assert_eq!(solve("cosh(0)"), 1.0);
    assert_eq!(solve("cosh(1)"), round(1_f64.cosh()));
    assert_eq!(solve("cosh(-1)"), round(1_f64.cosh())); // cosh is even
}
#[test]
fn tanh() {
    assert_eq!(solve("tanh(0)"), 0.0);
    assert_eq!(solve("tanh(1)"), round(1_f64.tanh()));
    assert_eq!(solve("tanh(-1)"), round(-1_f64.tanh()));
}

// trig identities
#[test]
fn pythagorean_identity() {
    // sin²(x) + cos²(x) = 1
    assert_eq!(solve("sin(pi/3)^2 + cos(pi/3)^2"), 1.0);
    assert_eq!(solve("sin(pi/7)^2 + cos(pi/7)^2"), 1.0);
    assert_eq!(solve("sin(1)^2 + cos(1)^2"), 1.0);
}

#[test]
fn trig_inverse_roundtrip() {
    // asin(sin(x)) = x for x in [-pi/2, pi/2]
    assert_eq!(solve("asin(sin(0.5))"), 0.5);
    assert_eq!(solve("acos(cos(0.5))"), 0.5);
    assert_eq!(solve("atan(tan(0.5))"), 0.5);
}

#[test]
fn log_exp_inverse() {
    assert_eq!(solve("ln(exp(3))"), 3.0);
    assert_eq!(solve("exp(ln(5))"), 5.0);
    assert_eq!(solve("log(10 ^ 4)"), 4.0);
}

#[test]
fn sqrt_pow_inverse() {
    assert_eq!(solve("sqrt(4) ^ 2"), 4.0);
    assert_eq!(solve("sqrt(9) ^ 2"), 9.0);
    assert_eq!(solve("cbrt(8) ^ 3"), 8.0);
}

#[test]
fn functions_composed() {
    assert_eq!(solve("abs(floor(-3.5))"), 4.0);
    assert_eq!(solve("max(abs(-5), abs(-3))"), 5.0);
    assert_eq!(solve("min(ceil(2.1), floor(3.9))"), 3.0);
    assert_eq!(solve("sqrt(pow(3, 2) + pow(4, 2))"), 5.0); // pythagorean
}
