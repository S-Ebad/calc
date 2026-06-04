use calc::calc::Calculator;

// helpers
fn solve(expr: &str) -> f64 {
    Calculator::new().solve(expr).expect(expr).expect("Doesn't return anything")
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
    fn large_factorial_overflow() {
        // The implementation caps factorial at 170
        assert!(solve_err("171!").contains("too large"));
    }

    #[test]
    fn modulo_with_decimals() {
        // 10.5 % 3 = 1.5
        assert_eq!(solve("10.5 % 3"), 1.5);
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
    fn scientific_notation_vs_euler() {
        assert_eq!(solve("9e"), round(9.0 * std::f64::consts::E));
        assert_eq!(solve("9e2"), 900.0);
        assert!(solve_err("9e9e9").contains("Invalid Number"));
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
        assert_eq!(solve("pi * 2"), round(2.0 * std::f64::consts::PI));
        assert_eq!(solve("e ^ 2"), round(std::f64::consts::E.powi(2)));
        assert_eq!(solve("pi / pi"), 1.0);
        assert_eq!(solve("e / e"), 1.0);
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
        assert_eq!(calc.solve("x + 3").unwrap().unwrap(), 8.0);
        assert_eq!(calc.solve("x * 2").unwrap().unwrap(), 10.0);
        assert_eq!(calc.solve("x ^ 2").unwrap().unwrap(), 25.0);
    }

    #[test]
    fn multiple_assignments() {
        let mut calc = Calculator::new();
        calc.solve("a = 3").unwrap();
        calc.solve("b = 4").unwrap();
        assert_eq!(calc.solve("a + b").unwrap().unwrap(), 7.0);
        assert_eq!(calc.solve("a * b").unwrap().unwrap(), 12.0);
    }

    #[test]
    fn reassignment() {
        let mut calc = Calculator::new();
        calc.solve("x = 5").unwrap();
        calc.solve("x = 10").unwrap();
        assert_eq!(calc.solve("x").unwrap().unwrap(), 10.0);
    }

    #[test]
    fn ans_updates() {
        let mut calc = Calculator::new();
        calc.solve("10 + 5").unwrap();
        assert_eq!(calc.solve("ans").unwrap().unwrap(), 15.0);
        calc.solve("3 * 3").unwrap();
        assert_eq!(calc.solve("ans").unwrap().unwrap(), 9.0);
    }

    #[test]
    fn ans_chaining() {
        let mut calc = Calculator::new();
        calc.solve("10").unwrap();
        assert_eq!(calc.solve("ans * 2").unwrap().unwrap(), 20.0);
        assert_eq!(calc.solve("ans + 5").unwrap().unwrap(), 25.0);
    }

    #[test]
    fn ans_uninitialized_error() {
        assert!(solve_err("ans").contains("ans not yet defined"));
    }

    #[test]
    fn unknown_variable_error() {
        assert!(solve_err("foo").contains("Unknown identifier"));
        assert!(solve_err("bar").contains("Unknown identifier"));
        assert!(solve_err("xyz").contains("Unknown identifier"));
    }

    #[test]
    fn variable_in_complex_expr() {
        let mut calc = Calculator::new();
        calc.solve("x = 3").unwrap();
        calc.solve("y = 4").unwrap();
        assert_eq!(round(calc.solve("sqrt(x^2 + y^2)").unwrap().unwrap()), 5.0); // pythagorean
        assert_eq!(calc.solve("2x + 3y").unwrap().unwrap(), 18.0); // implicit mul with vars
        assert_eq!(calc.solve("(x + y) ^ 2").unwrap().unwrap(), 49.0);
    }

    #[test]
    fn variable_reassign_affects_expr() {
        let mut calc = Calculator::new();
        calc.solve("x = 2").unwrap();
        assert_eq!(calc.solve("x ^ 3").unwrap().unwrap(), 8.0);
        calc.solve("x = 3").unwrap();
        assert_eq!(calc.solve("x ^ 3").unwrap().unwrap(), 27.0);
    }

    #[test]
    fn ans_persists_across_errors() {
        let mut calc = Calculator::new();
        calc.solve("10").unwrap();
        let _ = calc.solve("1 / 0"); // error shouldn't wipe ans
        assert_eq!(calc.solve("ans").unwrap().unwrap(), 10.0);
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

        assert!(solve_err("atan2(1)").contains("argument"));
        assert!(solve_err("atan2(1, 2, 3)").contains("argument"));
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
        assert!(solve_err("(1 + 2").contains("Missing closing parenthesis"));
        assert!(solve_err("((3 + 4)").contains("Missing closing parenthesis"));
    }

    #[test]
    fn mismatched_close() {
        assert!(solve_err("1 + 2)").contains("unexpected closing parenthesis"));
        assert!(solve_err("(1 + 2))").contains("unexpected closing parenthesis"));
    }

    #[test]
    fn empty_expr() {
        assert!(solve_err("").contains("No expression"));
    }

    #[test]
    fn invalid_token() {
        assert!(solve_err("1 @ 2").contains("Invalid Token"));
        assert!(solve_err("1 # 2").contains("Invalid Token"));
        assert!(solve_err("1 $ 2").contains("Invalid Token"));
    }

    #[test]
    fn missing_operator() {
        assert!(solve_err("1 2").contains("Missing operator"));
        assert!(solve_err("3 4 5").contains("Missing operator"));
    }

    #[test]
    fn tan_asymptote() {
        assert!(solve_is_err("tan(pi / 2)"));
        assert!(solve_is_err("tan(3 * pi / 2)"));
    }

    #[test]
    fn sqrt_negative() {
        assert!(solve_is_err("sqrt(-1)"));
    }

    #[test]
    fn ln_non_positive() {
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

    #[test]
    fn invalid_function_application() {
        let mut calc = Calculator::new();
        calc.solve("x = 5").unwrap();
        // Cannot multiply a variable by a list of arguments as if it were a function
        assert!(calc.solve("x(1, 2)").is_err());
    }

    #[test]
    fn invalid_function() {
        let mut calc = Calculator::new();

        // Func not yet defined
        assert!(calc.solve("5 + Func(5)").is_err())
    }
}

mod validation_tests {
    use super::*;

    #[test]
    fn invalid_redefinition() {
        let mut calc = Calculator::new();
        
        //ans is a reserved keyword
        assert!(calc.solve("ans = 50").is_err());

        // constants are reserved 
        assert!(calc.solve("pi = 13").is_err());
        assert!(calc.solve("e = 13").is_err());
        assert!(calc.solve("inf = 13").is_err());


        //sin is a built-in global
        assert!(calc.solve("sin(x) = 50x").is_err());
        assert!(calc.solve("cos(x, y, z) = 50x").is_err());

        // u can't make functions variables. It'll just break during parsing
        assert!(calc.solve("sin = 10").is_err());

        // same with user-defined functions
        calc.solve("f(x) = 2x").unwrap();
        assert!(calc.solve("f = 10").is_err());

        //weird quirk, u can't use defined functions as parameters
        calc.solve("g(x) = 2x").unwrap();
        //this is because parameters & arguments are parsed the same way. So when it sees "g" it
        //thinks It's a function & tries to parse it.
        assert!(calc.solve("h(g) = 2g").is_err());

        // this is fine with variables though
        calc.solve("x = 10").unwrap();
        calc.solve("f(x) = 10x").unwrap();
        assert_eq!(calc.solve("f(2)").unwrap().unwrap(), 20.0);
    }
}

mod user_function_tests {
    use super::*;

    #[test]
    fn multi_parameter_function() {
        let mut calc = Calculator::new();
        // Area of a cylinder: 2*pi*r*h + 2*pi*r^2
        calc.solve("cylinder_area(r, h) = 2pi*r*h + 2pi*r^2")
            .unwrap();
        let result = calc.solve("cylinder_area(3, 5)").unwrap().unwrap();
        let expected = round(
            2.0 * std::f64::consts::PI * 3.0 * 5.0 + 2.0 * std::f64::consts::PI * f64::powi(3.0, 2),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn function_redefinition() {
        let mut calc = Calculator::new();
        calc.solve("f(x) = x + 1").unwrap();
        assert_eq!(calc.solve("f(5)").unwrap().unwrap(), 6.0);

        // Redefine f(x)
        calc.solve("f(x) = x * 2").unwrap();
        assert_eq!(calc.solve("f(5)").unwrap().unwrap(), 10.0);
    }

    #[test]
    fn nested_user_calls() {
        let mut calc = Calculator::new();
        calc.solve("square(x) = x * x").unwrap();
        calc.solve("double(x) = x + x").unwrap();
        // double(square(3)) = (3*3) + (3*3) = 18
        assert_eq!(calc.solve("double(square(3))").unwrap().unwrap(), 18.0);
    }

    #[test]
    fn function_using_variables() {
        let mut calc = Calculator::new();
        calc.solve("k = 2").unwrap();
        calc.solve("f(x) = k * x").unwrap();
        assert_eq!(calc.solve("f(10)").unwrap().unwrap(), 20.0);

        // Changing k should change the result of the function call
        calc.solve("k = 5").unwrap();
        assert_eq!(calc.solve("f(10)").unwrap().unwrap(), 50.0);
    }

    #[test]
    fn arity_mismatch() {
        let mut calc = Calculator::new();
        calc.solve("f(x, y) = x + y").unwrap();
        // Too few arguments
        assert!(calc.solve("f(1)").is_err());
        // Too many arguments
        assert!(calc.solve("f(1, 2, 3)").is_err());
    }

    #[test]
    fn illegal_parameter_definition() {
        // Parameters must be identifiers, not numbers or expressions
        assert!(solve_err("f(10) = 10 * 2").contains("must be an identifier"));
    }

    #[test]
    fn implicit_mul() {
        let mut calc = Calculator::new();
        calc.solve("f(x) = 2x").unwrap();

        assert_eq!(calc.solve("f5").unwrap().unwrap(), 10.0);
        assert_eq!(calc.solve("2f5").unwrap().unwrap(), 20.0);
    }

    #[test]
    fn test_function_composition() {
        let mut calc = Calculator::new();
        calc.solve("f(x) = 10x").unwrap();
        calc.solve("g(x) = 20x").unwrap();

        assert_eq!(calc.solve("f(g(10))").unwrap().unwrap(), 2000.0);
    }
}
