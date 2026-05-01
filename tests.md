# Tests

Run the test suite with:

```
cargo test
```

## Test Modules

### Arithmetic
Covers the core operators (`+`, `-`, `*`, `/`, `%`, `^`), operator precedence, parentheses, right-associativity of `^`, floating-point precision (e.g. `0.1 + 0.2 = 0.3`), and chained/mixed operations.

### Unary
Covers unary `+` and `-`, double negation, factorial (`!`) for valid inputs (including `0!`), and error cases for factorial on negative numbers and non-integers.

### Constants
Covers `pi`, `e`, and `inf` in expressions, implicit multiplication (`2pi`, `2e`), implicit function calls (`sin pi`), and `inf` arithmetic (including the `inf - inf` NaN error).

### Variable
Covers variable assignment, reassignment, multiple variables in one expression, and the `ans` variable persisting across calls.

### Function
Covers all built-in functions including rounding, logs, trig, inverse trig, hyperbolic trig, angle conversion, `sqrt`/`cbrt`/`pow`, `max`/`min`/`clamp`/`gcd`/`lcm`, and `recip`. Also includes identity checks (Pythagorean identity, inverse round-trips like `asin(sin(x))`, `ln(exp(x))`) and composed function calls.

### Error
Covers expected error conditions: division by zero, `0^negative`, `recip(0)`, mismatched parentheses, empty input, invalid tokens (`@`, `#`, `$`), missing operators, `tan` at asymptotes, `sqrt(-1)`, `ln(-1)`, and invalid implicit multiplication with constants (e.g. `pi2`).

### User Functions
Covers user-defined function behavior including definition, invocation, multiple parameters, function redefinition, nested calls, interaction with variables, arity checking, invalid parameter definitions, and function composition. 

### Validation
Covers the reserved identifiers of the engine, ensuring check_errors() blocks re-definition of reserved keywords (ans), constants (pi, e), and built-in functions (sin, sqrt).

