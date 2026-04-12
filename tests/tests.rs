use calc::calc::Calculator;

// helpers
fn solve(expr: &str) -> f64 {
    Calculator::new().solve(expr).expect(expr)
}

fn solve_is_err(expr: &str) -> bool {
    Calculator::new().solve(expr).is_err()
}

fn solve_err(expr: &str) -> String {
    Calculator::new().solve(expr).expect_err(expr)
}

fn round(n: f64) -> f64 {
    (n * 1e10).round() / 1e10
}

mod arithmetic_tests {
    use super::*;

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
}

mod unary_tests {
    use super::*;

    #[test]
    fn unary_neg() {
        assert_eq!(solve("-5"), -5.0);
        assert_eq!(solve("-0"), 0.0);
        assert_eq!(solve("-100"), -100.0);
        assert_eq!(solve("--5"), 5.0);
    }
    #[test]
    fn unary_neg_in_expr() {
        assert_eq!(solve("10 + -3"), 7.0);
        assert_eq!(solve("10 - -3"), 13.0);
        assert_eq!(solve("5 * -2"), -10.0);
    }
    #[test]
    fn unary_pos() {
        assert_eq!(solve("+5"), 5.0);
        assert_eq!(solve("+0"), 0.0);
        assert_eq!(solve("+100"), 100.0);
    }

    #[test]
    fn factorial_5() {
        assert_eq!(solve("5!"), 120.0);
        assert_eq!(solve("3!"), 6.0);
        assert_eq!(solve("6!"), 720.0);
        assert_eq!(solve("10!"), 3628800.0);
    }
    #[test]
    fn factorial_0() {
        assert_eq!(solve("0!"), 1.0);
        assert_eq!(solve("1!"), 1.0);
        assert_eq!(solve("2!"), 2.0);
    }

    #[test]
    fn factorial_negative_is_error() {
        assert!(solve_err("(-1)!").contains("factorial undefined"));
        assert!(solve_err("(-5)!").contains("factorial undefined"));
    }

    #[test]
    fn factorial_non_integer_is_error() {
        assert!(solve_err("2.5!").contains("factorial undefined"));
        assert!(solve_err("0.1!").contains("factorial undefined"));
    }

    #[test]
    fn factorial_in_expr() {
        assert_eq!(solve("5! + 1"), 121.0);
        assert_eq!(solve("2 * 3!"), 12.0);
        assert_eq!(solve("3! ^ 2"), 36.0);
        assert_eq!(solve("(2 + 1)!"), 6.0);
    }

    #[test]
    fn unary_neg_with_functions() {
        assert_eq!(solve("abs -5"), 5.0);
        assert_eq!(solve("abs -100"), 100.0);
        assert_eq!(solve("floor -3.2"), -4.0);
        assert_eq!(solve("ceil -3.9"), -3.0);
    }

    #[test]
    fn double_negation() {
        assert_eq!(solve("--5"), 5.0);
        assert_eq!(solve("--0"), 0.0);
        assert_eq!(solve("1 + --3"), 4.0);
    }
}

mod constants_tests {
    use super::*;

    #[test]
    fn constants() {
        assert_eq!(solve("pi"), round(std::f64::consts::PI));
        assert_eq!(solve("e"), round(std::f64::consts::E));
        assert_eq!(solve("inf"), f64::INFINITY);
        assert_eq!(solve("-inf"), f64::NEG_INFINITY);
    }

    #[test]
    fn implicit_mul() {
        assert_eq!(solve("2(3 + 1)"), 8.0);
        assert_eq!(solve("(2 + 3)4"), 20.0);
        assert_eq!(solve("2pi"), round(2.0 * std::f64::consts::PI));
        assert_eq!(solve("3(2 + 2)"), 12.0);
        assert_eq!(solve("2e"), round(2.0 * std::f64::consts::E));
    }

    #[test]
    fn implicit_function_call() {
        assert_eq!(solve("sin pi"), 0.0);
        assert_eq!(solve("cos pi"), -1.0);
        assert_eq!(solve("abs -5"), 5.0);
    }

    #[test]
    fn implicit_chain() {
        assert_eq!(solve("pi sin pi"), 0.0);
        assert_eq!(solve("2 cos pi"), -2.0);
    }

    #[test]
    fn constants_in_expressions() {
        assert_eq!(round(solve("pi * 2")), round(2.0 * std::f64::consts::PI));
        assert_eq!(round(solve("e ^ 2")), round(std::f64::consts::E.powi(2)));
        assert_eq!(round(solve("pi / pi")), 1.0);
        assert_eq!(round(solve("e / e")), 1.0);
    }

    #[test]
    fn inf_arithmetic() {
        assert_eq!(solve("inf + 1"), f64::INFINITY);
        assert_eq!(solve("inf * 2"), f64::INFINITY);
        assert_eq!(solve("-inf - 1"), f64::NEG_INFINITY);
        assert!(solve_err("inf - inf").contains("NaN"));
    }
}

mod variable_tests {
    use super::*;

    #[test]
    fn assignment() {
        let mut calc = Calculator::new();
        calc.solve("x = 5").unwrap();
        assert_eq!(calc.solve("x + 3").unwrap(), 8.0);
        assert_eq!(calc.solve("x * 2").unwrap(), 10.0);
        assert_eq!(calc.solve("x ^ 2").unwrap(), 25.0);
    }

    #[test]
    fn multiple_assignments() {
        let mut calc = Calculator::new();
        calc.solve("a = 3").unwrap();
        calc.solve("b = 4").unwrap();
        assert_eq!(calc.solve("a + b").unwrap(), 7.0);
        assert_eq!(calc.solve("a * b").unwrap(), 12.0);
    }

    #[test]
    fn reassignment() {
        let mut calc = Calculator::new();
        calc.solve("x = 5").unwrap();
        calc.solve("x = 10").unwrap();
        assert_eq!(calc.solve("x").unwrap(), 10.0);
    }

    #[test]
    fn ans_updates() {
        let mut calc = Calculator::new();
        calc.solve("10 + 5").unwrap();
        assert_eq!(calc.solve("ans").unwrap(), 15.0);
        calc.solve("3 * 3").unwrap();
        assert_eq!(calc.solve("ans").unwrap(), 9.0);
    }

    #[test]
    fn ans_chaining() {
        let mut calc = Calculator::new();
        calc.solve("10").unwrap();
        assert_eq!(calc.solve("ans * 2").unwrap(), 20.0);
        assert_eq!(calc.solve("ans + 5").unwrap(), 25.0);
    }

    #[test]
    fn ans_uninitialized_error() {
        assert!(solve_err("ans").contains("ans is not yet defined"));
    }

    #[test]
    fn unknown_variable_error() {
        assert!(solve_err("foo").contains("Invalid Identifier"));
        assert!(solve_err("bar").contains("Invalid Identifier"));
        assert!(solve_err("xyz").contains("Invalid Identifier"));
    }

    #[test]
    fn variable_in_complex_expr() {
        let mut calc = Calculator::new();
        calc.solve("x = 3").unwrap();
        calc.solve("y = 4").unwrap();
        assert_eq!(round(calc.solve("sqrt(x^2 + y^2)").unwrap()), 5.0); // pythagorean
        assert_eq!(calc.solve("2x + 3y").unwrap(), 18.0); // implicit mul with vars
        assert_eq!(calc.solve("(x + y) ^ 2").unwrap(), 49.0);
    }

    #[test]
    fn variable_reassign_affects_expr() {
        let mut calc = Calculator::new();
        calc.solve("x = 2").unwrap();
        assert_eq!(calc.solve("x ^ 3").unwrap(), 8.0);
        calc.solve("x = 3").unwrap();
        assert_eq!(calc.solve("x ^ 3").unwrap(), 27.0);
    }

    #[test]
    fn ans_persists_across_errors() {
        let mut calc = Calculator::new();
        calc.solve("10").unwrap();
        let _ = calc.solve("1 / 0"); // error shouldn't wipe ans
        assert_eq!(calc.solve("ans").unwrap(), 10.0);
    }
}

mod function_tests {
    use super::*;

    // trig
    #[test]
    fn sin() {
        assert_eq!(solve("sin(0)"), 0.0);
        assert_eq!(solve("sin(pi)"), 0.0);
        assert_eq!(solve("sin(pi / 2)"), 1.0);
        assert_eq!(round(solve("sin(pi / 6)")), round(0.5));
    }
    #[test]
    fn cos() {
        assert_eq!(solve("cos(0)"), 1.0);
        assert_eq!(solve("cos(pi)"), -1.0);
        assert_eq!(round(solve("cos(pi / 2)")), 0.0);
        assert_eq!(round(solve("cos(pi / 3)")), round(0.5));
    }
    #[test]
    fn tan() {
        assert_eq!(solve("tan(0)"), 0.0);
        assert_eq!(round(solve("tan(pi / 4)")), 1.0);
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
        assert_eq!(round(solve("ln(e ^ 2)")), 2.0);
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
        assert_eq!(round(solve("exp(2)")), round(std::f64::consts::E.powi(2)));
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
        assert_eq!(round(solve("deg(pi / 2)")), 90.0);
    }
    #[test]
    fn rad() {
        assert_eq!(solve("rad(180)"), round(std::f64::consts::PI));
        assert_eq!(solve("rad(0)"), 0.0);
        assert_eq!(round(solve("rad(360)")), round(2.0 * std::f64::consts::PI));
        assert_eq!(round(solve("rad(90)")), round(std::f64::consts::FRAC_PI_2));
    }

    // trig inverse
    #[test]
    fn asin() {
        assert_eq!(solve("asin(1)"), round(std::f64::consts::FRAC_PI_2));
        assert_eq!(solve("asin(0)"), 0.0);
        assert_eq!(
            round(solve("asin(-1)")),
            round(-std::f64::consts::FRAC_PI_2)
        );
    }
    #[test]
    fn acos() {
        assert_eq!(solve("acos(1)"), 0.0);
        assert_eq!(solve("acos(0)"), round(std::f64::consts::FRAC_PI_2));
        assert_eq!(round(solve("acos(-1)")), round(std::f64::consts::PI));
    }
    #[test]
    fn atan() {
        assert_eq!(solve("atan(1)"), round(std::f64::consts::FRAC_PI_4));
        assert_eq!(solve("atan(0)"), 0.0);
        assert_eq!(
            round(solve("atan(-1)")),
            round(-std::f64::consts::FRAC_PI_4)
        );
    }

    // hyperbolic
    #[test]
    fn sinh() {
        assert_eq!(solve("sinh(0)"), 0.0);
        assert_eq!(solve("sinh(1)"), round(1_f64.sinh()));
        assert_eq!(round(solve("sinh(-1)")), round(-1_f64.sinh()));
    }
    #[test]
    fn cosh() {
        assert_eq!(solve("cosh(0)"), 1.0);
        assert_eq!(solve("cosh(1)"), round(1_f64.cosh()));
        assert_eq!(round(solve("cosh(-1)")), round(1_f64.cosh())); // cosh is even
    }
    #[test]
    fn tanh() {
        assert_eq!(solve("tanh(0)"), 0.0);
        assert_eq!(solve("tanh(1)"), round(1_f64.tanh()));
        assert_eq!(round(solve("tanh(-1)")), round(-1_f64.tanh()));
    }

    // trig identities
    #[test]
    fn pythagorean_identity() {
        // sin²(x) + cos²(x) = 1
        assert_eq!(round(solve("sin(pi/3)^2 + cos(pi/3)^2")), 1.0);
        assert_eq!(round(solve("sin(pi/7)^2 + cos(pi/7)^2")), 1.0);
        assert_eq!(round(solve("sin(1)^2 + cos(1)^2")), 1.0);
    }

    #[test]
    fn trig_inverse_roundtrip() {
        // asin(sin(x)) = x for x in [-pi/2, pi/2]
        assert_eq!(round(solve("asin(sin(0.5))")), 0.5);
        assert_eq!(round(solve("acos(cos(0.5))")), 0.5);
        assert_eq!(round(solve("atan(tan(0.5))")), 0.5);
    }

    #[test]
    fn log_exp_inverse() {
        assert_eq!(round(solve("ln(exp(3))")), 3.0);
        assert_eq!(round(solve("exp(ln(5))")), 5.0);
        assert_eq!(round(solve("log(10 ^ 4)")), 4.0);
    }

    #[test]
    fn sqrt_pow_inverse() {
        assert_eq!(solve("sqrt(4) ^ 2"), 4.0);
        assert_eq!(solve("sqrt(9) ^ 2"), 9.0);
        assert_eq!(round(solve("cbrt(8) ^ 3")), 8.0);
    }

    #[test]
    fn functions_composed() {
        assert_eq!(solve("abs(floor(-3.5))"), 4.0);
        assert_eq!(solve("max(abs(-5), abs(-3))"), 5.0);
        assert_eq!(solve("min(ceil(2.1), floor(3.9))"), 3.0);
        assert_eq!(round(solve("sqrt(pow(3, 2) + pow(4, 2))")), 5.0); // pythagorean
    }
}

mod error_tests {
    use super::*;

    #[test]
    fn division_by_zero() {
        assert!(solve_err("1 / 0").contains("division by zero"));
        assert!(solve_err("100 / 0").contains("division by zero"));
        assert!(solve_err("-5 / 0").contains("division by zero"));
    }

    #[test]
    fn power_zero_to_negative() {
        assert!(solve_err("0 ^ -1").contains("division by zero"));
        assert!(solve_err("0 ^ -2").contains("division by zero"));
        assert!(solve_err("pow(0, -1)").contains("division by zero"));
        assert!(solve_err("pow(0, -5)").contains("division by zero"));
    }

    #[test]
    fn recip_zero() {
        assert!(solve_err("recip(0)").contains("division by zero"));
    }

    #[test]
    fn mismatched_open() {
        assert!(solve_err("(1 + 2").contains("unclosed parentheses"));
        assert!(solve_err("((3 + 4)").contains("unclosed parentheses"));
    }

    #[test]
    fn mismatched_close() {
        assert!(solve_err("1 + 2)").contains("mismatched parentheses"));
        assert!(solve_err("(1 + 2))").contains("mismatched parentheses"));
    }

    #[test]
    fn empty_expr() {
        assert!(solve_err("").contains("no expression"));
    }

    #[test]
    fn invalid_token() {
        assert!(solve_err("1 @ 2").contains("Invalid Token"));
        assert!(solve_err("1 # 2").contains("Invalid Token"));
        assert!(solve_err("1 $ 2").contains("Invalid Token"));
    }

    #[test]
    fn missing_operator() {
        assert!(solve_err("1 2").contains("missing operator"));
        assert!(solve_err("3 4 5").contains("missing operator"));
    }

    #[test]
    fn tan_asymptote() {
        assert!(solve_is_err("tan(pi / 2)"));
        assert!(solve_is_err("tan(3 * pi / 2)"));
    }

    #[test]
    fn constant_implicit_error() {
        assert!(solve_err("pi2").contains("isn't supported"));
        assert!(solve_err("e2").contains("isn't supported"));
    }

    #[test]
    fn sqrt_negative() {
        assert!(solve_is_err("sqrt(-1)"));
    }

    #[test]
    fn ln_non_positive() {
        // assert_eq!(solev())
        // assert!(solve_is_err("ln(0)"));
        assert!(solve_is_err("ln(-1)"));
    }

    #[test]
    fn log_non_positive() {
        assert_eq!(solve("log(0)"), f64::NEG_INFINITY);
        assert!(solve_is_err("log(-1)"));
    }

    #[test]
    fn deeply_nested_parens() {
        // valid deep nesting shouldn't error
        assert_eq!(solve("((((((1 + 2))))))"), 3.0);
    }

    #[test]
    fn whitespace_handling() {
        assert_eq!(solve("1+2"), 3.0);
        assert_eq!(solve("1   +   2"), 3.0);
        assert_eq!(solve("  10  /  2  "), 5.0);
    }
}
