# Tests

Run the test suite with:

```
cargo test
```

## Test Modules

### arithmetic_tests
Covers the core operators (`+`, `-`, `*`, `/`, `%`, `^`), operator precedence, parentheses, right-associativity of `^`, floating-point precision (e.g. `0.1 + 0.2 = 0.3`), and chained/mixed operations.

### unary_tests
Covers unary `+` and `-`, double negation, factorial (`!`) for valid inputs (including `0!`), and error cases for factorial on negative numbers and non-integers.

### constants_tests
Covers `pi`, `e`, and `inf` in expressions, implicit multiplication (`2pi`, `2e`), implicit function calls (`sin pi`), and `inf` arithmetic (including the `inf - inf` NaN error).

### variable_tests
Covers variable assignment, reassignment, multiple variables in one expression, and the `ans` variable persisting across calls.

### function_tests
Covers all built-in functions including rounding, logs, trig, inverse trig, hyperbolic trig, angle conversion, `sqrt`/`cbrt`/`pow`, `max`/`min`/`clamp`/`gcd`/`lcm`, and `recip`. Also includes identity checks (Pythagorean identity, inverse round-trips like `asin(sin(x))`, `ln(exp(x))`) and composed function calls.

### error_tests
Covers expected error conditions: division by zero, `0^negative`, `recip(0)`, mismatched parentheses, empty input, invalid tokens (`@`, `#`, `$`), missing operators, `tan` at asymptotes, `sqrt(-1)`, `ln(-1)`, and invalid implicit multiplication with constants (e.g. `pi2`).
