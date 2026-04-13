# Features

## Operators

| Operator | Description                  |
|----------|------------------------------|
| `+`      | Addition                     |
| `-`      | Subtraction / unary negation |
| `*`      | Multiplication               |
| `/`      | Division                     |
| `%`      | Modulo                       |
| `^`      | Exponentiation (right-associative) |
| `!`      | Factorial (postfix)          |
| `=`      | Variable assignment          |

## Constants

| Name  | Value       |
|-------|-------------|
| `pi`  | π           |
| `e`   | Euler's number |
| `inf` | Infinity    |

## Variables

Assign with `=`. The special variable `ans` always holds the last result.

```
> x = 5
= 5
> x * 2
= 10
> ans + 1
= 11
```

## Functions

| Function              | Description                                        |
|-----------------------|----------------------------------------------------|
| `sin(x)`              | Sine (radians)                                     |
| `cos(x)`              | Cosine (radians)                                   |
| `tan(x)`              | Tangent (radians); errors at asymptotes            |
| `asin(x)`             | Arcsine                                            |
| `acos(x)`             | Arccosine                                          |
| `atan(x)`             | Arctangent                                         |
| `sinh(x)`             | Hyperbolic sine                                    |
| `cosh(x)`             | Hyperbolic cosine                                  |
| `tanh(x)`             | Hyperbolic tangent                                 |
| `sqrt(x)`             | Square root; `sqrt(x, n)` for nth root             |
| `cbrt(x)`             | Cube root                                          |
| `pow(x, n)`           | x raised to the power n                            |
| `log(x)`              | Log base 10; `log(x, b)` for arbitrary base        |
| `ln(x)`               | Natural log                                        |
| `exp(x)`              | e^x                                                |
| `abs(x)`              | Absolute value                                     |
| `floor(x)`            | Round down                                         |
| `ceil(x)`             | Round up                                           |
| `round(x)`            | Round to nearest                                   |
| `trunc(x)` / `int(x)` | Truncate decimals toward zero                      |
| `recip(x)`            | 1/x                                                |
| `clamp(x, min, max)`  | Clamp value to range                               |
| `max(a, b, ...)`      | Maximum of two or more arguments                   |
| `min(a, b, ...)`      | Minimum of two or more arguments                   |
| `gcd(a, b)`           | Greatest common divisor                            |
| `lcm(a, b)`           | Least common multiple                              |
| `rad(x)`              | Degrees to radians                                 |
| `deg(x)`              | Radians to degrees                                 |

## Implicit Syntax

Parentheses and function calls can often be omitted for brevity.

**Implicit multiplication**: a number or closing paren followed by an identifier or opening paren:
```
2pi        → 2 * pi
3(x + 1)   → 3 * (x + 1)
(2 + 3)4   → (2 + 3) * 4
```

**Implicit function calls**: a function followed directly by a number or constant (wraps a single token):
```
sin pi     → sin(pi)
sqrt2      → sqrt(2)
abs -5     → abs(-5)
```
