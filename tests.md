# Tests

Run the test suite with:

```
cargo test
```

## Test Modules

### Arithmetic
Covers the core operators (`+`, `-`, `*`, `/`, `%`, `^`), operator precedence, parentheses, right-associativity of `^`, floating-point precision (e.g. `0.1 + 0.2 = 0.3`), repeated addition precision, chained/mixed operations, large numbers, negative results, nth roots via `sqrt(x, n)`, modulo with decimals, and factorial overflow.

### Unary
Covers unary `+` and `-`, double negation (`--5`), unary operators inside expressions (`10 + -3`, `10 - -3`), and unary negation combined with built-in functions (`abs -5`, `floor -3.2`).

### Postfix
Covers factorial (`!`) for valid inputs including `0!` and `1!`, error cases for factorial on negatives and non-integers, and factorial inside larger expressions.

### Constants
Covers `pi`, `e`, `inf`, `true`, and `false` in expressions, implicit multiplication (`2pi`, `2e`, `2true`, `5false`), scientific notation vs Euler's number disambiguation (`9e` vs `9e2`), implicit function calls (`sin pi`), implicit chaining (`pi sin pi`), and `inf` arithmetic including the `inf - inf` NaN error.

### Variable
Covers variable assignment, reassignment, multiple variables in one expression, the `ans` variable persisting across calls, `ans` chaining, uninitialized `ans` error, unknown variable errors, variables in complex expressions (implicit mul, Pythagorean), and error isolation (`ans` not wiped on failed eval).

### Function
Covers all built-in functions: sin, cos, tan, sqrt, nth root, atan2, abs, floor, ceil, round, int, etc (check [features](features.md) for more). Also covers Pythagorean identity, inverse round-trips (asin(sin(x)), ln(exp(x))), and composed function calls

### Logical Operators
Covers equality (`==`, `!=`), comparison (`<`, `>`, `<=`, `>=`), boolean logic (`&&`, `||`) with `true`/`false` and numeric truthiness, and operator precedence (arithmetic > comparison > logical).

### Ternary
Covers basic true/false branch selection, ternary with comparison conditions, nested ternary expressions, and complex expressions in branches including arithmetic and function calls.

### Error
Covers expected error conditions: division by zero, `0^negative`, `recip(0)`, mismatched parentheses, empty input, invalid tokens (`@`, `#`, `$`), missing operators, `tan` at asymptotes, `sqrt(-1)`, `ln(-1)`, invalid multi-argument applicatio on variable, and calling an undefined function.

### Validation
Covers `check_errors()`, blocks redefinition of ans, constants (pi, e, inf, true, false), and built-in functions (sin, cos). Also covers the parameter shadowing restriction (user-defined function names cannot be used as parameters) and confirms variable names can be reused as parameters without issue.

### User Functions
Covers definition and invocation, multi-parameter functions, function redefinition, nested user function calls, functions referencing global variables, dynamic scoping (global variable changes affect function output at call time), arity mismatch errors (too few and too many args), illegal parameter definitions, implicit multiplication with user functions (`f5`, `2f5`), and function composition (`f(g(x))`).
