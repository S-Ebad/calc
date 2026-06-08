# Features

## Operators

| Operator | Description                        |
|----------|------------------------------------|
| `+`      | Addition                           |
| `-`      | Subtraction / unary negation       |
| `*`      | Multiplication                     |
| `/`      | Division                           |
| `%`      | Modulo                             |
| `^`      | Exponentiation (right-associative) |
| `!`      | Factorial (postfix)                |
| `=`      | Variable assignment                |
| `==`     | Is equal                           |
| `!=`     | Not equal                          |
| `<`      | Less than                          |
| `>`      | Greater than                       |
| `<=`     | Less than or equal to              |
| `>=`     | Greater than or equal to           |
| `&&`     | Logical AND                        |
| `||`     | Logical OR                         |

## Constants

| Name     | Value                              |
|----------|------------------------------------|
| `pi`     | π                                  |
| `e`      | Euler's number                     |
| `inf`    | Infinity                           |
| `true`   | Boolean true (evaluates to `1.0`)  |
| `false`  | Boolean false (evaluates to `0.0`) |


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

## Ternary
```
cond ? then : else
```

```
> 1 > 0 ? 10 : 20
= 10

> x = -5
> x >= 0 ? x : -x
= 5
```

## Variables

Assign with `=`. The special variable `ans` always holds the last result and is read-only.

```
> x = 5
= 5

> x * 2
= 10

> ans + 1
= 11

> ans = 10
Parse Error: 'ans' is a reserved read-only variable
```

## Implicit Syntax

Parentheses and function calls can often be omitted for brevity.

**Implicit multiplication**: a number or closing paren followed by an identifier or opening paren:
```
2pi        → 2 * pi
3(x + 1)   → 3 * (x + 1)
(2 + 3)4   → (2 + 3) * 4
```

**Implicit function calls**: a function followed directly by a number or identifier (wraps a single token):
```
sin pi     → sin(pi)
sqrt2      → sqrt(2)
abs -5     → abs(-5)
3sinpi     → 3 * sin(pi)
```
## User-defined Functions
define with `name(param1, param2, ...) = body`.

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

> g(0)
Resolver Error: Function g takes 2 argument(s) but got 1

> g(0,1,2)
Resolver Error: Function g takes 2 argument(s) but got 3
```

Functions use dynamic scoping, globals are resolved at call time, not definition time. Redefining a function replaces the previous definition.

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

> Recursive functions are supported with a depth limit of 100

```
> fac(x) = x > 2 ? fac(x-1) * x : x

> fac(10)
= 3628800
```

## Parameter Shadowing Restriction
Function parameters cannot share names with existing user-defined functions. This is because the parser eagerly identifies known function names when building the AST, so passing a function name as a parameter causes a parse error.
```
> f(x) = 2x

> g(f) = 2f
Parse Error: 'f' is a function, not a value
```
This restriction does not apply to variables, a variable name can be reused as a parameter without issue.

```
> f(x) = 2x

> x = 10

> f(x)
= 20
```
