use calc::calc::Calculator;

// helper functions

/// Solve an expression and return the f64 result, panicking on error.
fn solve(expr: &str) -> f64 {
    Calculator::new().solve(expr).expect(expr)
}

/// Solve and expect an error; returns the error message string.
fn solve_err(expr: &str) -> String {
    Calculator::new().solve(expr).expect_err(expr)
}

// used to round rust functions as the calculator rounds
fn round(n: f64) -> f64 {
    (n * 1e10).round() / 1e10
}

// basic arithmetic
#[test]
fn test_addition() {
    assert_eq!(solve("1 + 2"), 3.0);
}

#[test]
fn test_subtraction() {
    assert_eq!(solve("10 - 4"), 6.0);
}

#[test]
fn test_multiplication() {
    assert_eq!(solve("3 * 4"), 12.0);
}

#[test]
fn test_division() {
    assert_eq!(solve("10 / 4"), 2.5);
}

#[test]
fn test_modulo() {
    assert_eq!(solve("10 % 3"), 1.0);
}

#[test]
fn test_exponentiation() {
    assert_eq!(solve("2 ^ 10"), 1024.0);
}

#[test]
fn test_float_precision() {
    // 0.1 + 0.2 is a classic floating-point pitfall; the formatter handles it
    assert_eq!(solve("0.1 + 0.2"), 0.3);
}

// operator precedence
#[test]
fn test_precedence_mul_before_add() {
    assert_eq!(solve("2 + 3 * 4"), 14.0);
}

#[test]
fn test_parens_override_precedence() {
    assert_eq!(solve("(2 + 3) * 4"), 20.0);
}

#[test]
fn test_nested_parens() {
    assert_eq!(solve("((2 + 3) * (1 + 1))"), 10.0);
}

#[test]
fn test_right_assoc_power() {
    // 2^3^2 = 2^(3^2) = 2^9 = 512  (right-associative)
    assert_eq!(solve("2^3^2"), 512.0);
}

// unary test
#[test]
fn test_unary_neg() {
    assert_eq!(solve("-5"), -5.0);
}

#[test]
fn test_unary_neg_in_expr() {
    assert_eq!(solve("10 + -3"), 7.0);
}

#[test]
fn test_unary_pos() {
    assert_eq!(solve("+5"), 5.0);
}

// factorial
#[test]
fn test_factorial_5() {
    assert_eq!(solve("5!"), 120.0);
}

#[test]
fn test_factorial_0() {
    assert_eq!(solve("0!"), 1.0);
}

#[test]
fn test_factorial_negative_is_error() {
    assert!(solve_err("(-1)!").contains("factorial undefined"));
}

#[test]
fn test_factorial_non_integer_is_error() {
    assert!(solve_err("2.5!").contains("factorial undefined"));
}

// constants
#[test]
fn test_constant_pi() {
    assert_eq!(solve("pi"), round(std::f64::consts::PI));
}

#[test]
fn test_constant_e() {
    assert_eq!(solve("e"), round(std::f64::consts::E));
}

#[test]
fn test_constant_inf() {
    assert_eq!(solve("inf"), f64::INFINITY);
}

// implicit multiplication
#[test]
fn test_implicit_mul_number_paren() {
    assert_eq!(solve("2(3 + 1)"), 8.0);
}

#[test]
fn test_implicit_mul_number_constant() {
    assert_eq!(solve("2pi"), round(2.0 * std::f64::consts::PI));
}

#[test]
fn test_implicit_mul_paren_number() {
    assert_eq!(solve("(2 + 3)4"), 20.0);
}

#[test]
fn test_implicit_mul_function() {
    assert_eq!(solve("sin pi"), 0.0);
}

// variable
#[test]
fn test_variable_assignment_and_use() {
    let mut calc = Calculator::new();
    calc.solve("x = 5").unwrap();
    assert_eq!(calc.solve("x + 3").unwrap(), 8.0);
}

#[test]
fn test_ans_updated_after_solve() {
    let mut calc = Calculator::new();
    calc.solve("10 + 5").unwrap();
    assert_eq!(calc.solve("ans").unwrap(), 15.0);
}

#[test]
fn test_ans_used_in_next_expression() {
    let mut calc = Calculator::new();
    calc.solve("10").unwrap();
    assert_eq!(calc.solve("ans * 2").unwrap(), 20.0);
}

#[test]
fn test_ans_before_first_solve_is_error() {
    assert!(solve_err("ans").contains("ans is not yet defined"));
}

#[test]
fn test_unknown_variable_is_error() {
    assert!(solve_err("foo").contains("Invalid Identifier"));
}

// builtin functions
#[test]
fn test_fn_sin() {
    assert_eq!(solve("sin(0)"), 0.0);
}

#[test]
fn test_fn_cos() {
    assert_eq!(solve("cos(0)"), 1.0);
}

#[test]
fn test_fn_sqrt() {
    assert_eq!(solve("sqrt(9)"), 3.0);
}

#[test]
fn test_fn_sqrt_nth_root() {
    // cube root of 27
    assert_eq!(solve("sqrt(27, 3)"), 3.0);
}

#[test]
fn test_fn_abs_negative() {
    assert_eq!(solve("abs(-7)"), 7.0);
}

#[test]
fn test_fn_floor() {
    assert_eq!(solve("floor(3.9)"), 3.0);
}

#[test]
fn test_fn_ceil() {
    assert_eq!(solve("ceil(3.1)"), 4.0);
}

#[test]
fn test_fn_round() {
    assert_eq!(solve("round(3.5)"), 4.0);
}

#[test]
fn test_fn_ln() {
    assert_eq!(solve("ln(e)"), 1.0);
}

#[test]
fn test_fn_log_base10() {
    assert_eq!(solve("log(100)"), 2.0);
}

#[test]
fn test_fn_log_custom_base() {
    assert_eq!(solve("log(8, 2)"), 3.0);
}

#[test]
fn test_fn_exp() {
    assert_eq!(solve("exp(1)"), round(std::f64::consts::E));
}

#[test]
fn test_fn_pow() {
    assert_eq!(solve("pow(3, 4)"), 81.0);
}

#[test]
fn test_fn_max() {
    assert_eq!(solve("max(1, 5, 3)"), 5.0);
}

#[test]
fn test_fn_min() {
    assert_eq!(solve("min(4, 2, 9)"), 2.0);
}

#[test]
fn test_fn_cbrt() {
    // cube root of 27
    assert_eq!(solve("cbrt(27)"), 3.0);
}

#[test]
fn test_fn_recip() {
    assert_eq!(solve("recip(4)"), 0.25);
}

#[test]
fn test_fn_deg() {
    assert_eq!(solve("deg(pi)"), 180.0);
}

#[test]
fn test_fn_rad() {
    assert_eq!(solve("rad(180)"), round(std::f64::consts::PI));
}

#[test]
fn test_fn_implicit_call() {
    // sin pi without explicit parens
    assert_eq!(solve("sin pi"), 0.0);
}

#[test]
fn test_fn_implicit_call2() {
    assert_eq!(solve("pi sin pi"), 0.0);
}

// error cases
#[test]
fn test_division_by_zero() {
    assert!(solve_err("1 / 0").contains("division by zero"));
}

#[test]
fn test_power_zero_to_negative() {
    assert!(solve_err("0 ^ -1").contains("division by zero"));
}

#[test]
fn test_recip_zero() {
    assert!(solve_err("recip(0)").contains("division by zero"));
}

#[test]
fn test_mismatched_parens_open() {
    assert!(solve_err("(1 + 2").contains("unclosed parentheses"));
}

#[test]
fn test_mismatched_parens_close() {
    assert!(solve_err("1 + 2)").contains("mismatched parentheses"));
}

#[test]
fn test_empty_expression() {
    assert!(solve_err("").contains("no expression"));
}

#[test]
fn test_invalid_token() {
    assert!(solve_err("1 @ 2").contains("Invalid Token"));
}

#[test]
fn test_missing_operator() {
    assert!(solve_err("1 2").contains("missing operator"));
}

#[test]
fn test_tan_asymptote() {
    // tan(π/2) is undefined
    let result = Calculator::new().solve("tan(pi / 2)");
    assert!(result.is_err());
}

#[test]
fn test_constant_implicit_mul() {
    assert!(solve_err("pi2").contains("isn't supported"));
}
