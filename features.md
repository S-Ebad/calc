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

Assign with `=`. The special variable ans always holds the last result and is read-only. It cannot be manually reassigned.

```
> x = 5
= 5

> x * 2
= 10

> ans + 1
= 11
```

## Built-in Functions

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
3sinpi     → 3 * sin(pi)
```
## User-defined Functions
define with `name(param1, param2...) = body`.

```
> f(x) = x ^ 2 + 1

> f(4)
= 17

> f2
= 5

> 2f2
= 10

> g(x, y) = sqrt(x^2 + y^2)

> g(3, 4)
= 5

```

Functions can reference other user functions and variables defined at call time. Redefining function replaces the previous definition (function overloading not supported).
```
> f(x) = g(x)

> g(x) = 2x

> f(10)
= 20

> x = 10

> f(y) = x + g(y)

> f(10)
= 30
```


> Functions are inlined at call time. The body is substituted with argument values and then evaluated. There is a recursion depth limit of 100.

## Parameter Shadowing Restriction
Function parameters cannot share names with existing user-defined functions. This is due to the parser's eager identification of function symbols during the construction of the Abstract Syntax TreeFunction parameters cannot share names with existing user-defined functions. This is due to the parser's eager identification of function symbols during the construction of the Abstract Syntax Tree.

```
> f(x) = 2x

> g(f) = 2f
Error: Invalid Token: unexpected token RParen at start of expression
```



