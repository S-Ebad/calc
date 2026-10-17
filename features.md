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

Functions use dynamic scoping, globals are resolved at call time, not definition time, and only in the branch a call actually takes. A wrong number of arguments to a built-in is the exception: it can never become right, so it is rejected when the function is defined. Redefining a function replaces the previous definition.

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

## Parameter Shadowing
Inside a function's body, a parameter shadows anything of the same name: a variable, a user-defined function, or a built-in. The name refers to the value passed in, wherever it appears in the body.
```
> f(x) = 2x

> x = 10

> g(f) = f - 2*3

> g(x)
= 4

> f(x)
= 20
```
Outside the body nothing changes, so `f` is still the function. Constants (`pi`, `e`, ...) can't be used as parameters.

## UFCS (Uniform Function Call Syntax)
Any value can call a function as if it were a method, with the value passed as the first argument: `value.f(rest, of, args)` is equivalent to `f(value, rest, of, args)`.

Only functions can be called this way. With `x = 3`, `5.x()` is an error rather than `5 * 3`, and the same goes for a function's parameters inside its body.

```
> f(x) = 2x

> 5.f()
= 10

> g(x, y) = x + y

> 5.g(3)
= 8

> g(x) = x + 1

5.f().g()
= 11
```

The lexer is careful to distinguish a decimal point from a UFCS dot, `10.5.max(3)` parses `10.5` as the number, then `.max(3)` as a method call, rather than getting confused by the second `.`:
```
sqrt 10.5.max(3)
= 3.24
```


## Statements
Several statements can share one input, separated by `;`. They run in order, so each one sees what the ones before it defined, and the answer is the last statement's. Line breaks are just whitespace, which means a whole file of definitions can be passed in at once.

```
> a = 2; b = a * 3; a + b
= 8

> sq(x) = x^2; cube(x) = x * sq(x); cube(3)
= 27
```

Empty statements, such as a trailing `;`, are skipped. The first error stops the statements after it, and whatever ran before it stays defined. The error points into the whole input:

```
> a = 1; b = zzz; c = 3
Resolver Error: unknown identifier 'zzz'
 a = 1; b = zzz; c = 3
            ^^^
```

Each function keeps only its own statement as its definition, so `help` shows just that statement.

## Comments
`#` starts a comment, which runs to the end of the line. A `;` inside a comment is part of it, and a statement that is only a comment is skipped.

```
# std.math
sq(x) = x^2;          # square
cube(x) = x * sq(x);  # uses sq; defined above
```

A comment on the same line as a statement, after its `;`, goes with that statement; comments on the lines above a statement go with it too. A function keeps its comments with its definition, so `help` shows them:

```
> help cube
cube is a user-defined function.

cube(x) = x * sq(x);  # uses sq; defined above
```
