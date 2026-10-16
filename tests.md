# Tests

Run the test suite with:

```
cargo test
```

## Layout

Each file in `tests/` is its own test binary, so a failure in one area is
reported on its own rather than alongside everything else. Shared helpers live
in `tests/common/`, which is a directory so that it is not collected as a test
of its own; every file pulls it in with `mod common;`.

The helpers are a `Session` that keeps state across lines the way the REPL
does, `solve`/`render` for one-off expressions, and `lexer_kind`, `parse_kind`,
`resolver_kind` and `eval_kind` for reaching the typed kind behind an error.

## Files

### arithmetic
The core operators (`+`, `-`, `*`, `/`, `%`, `^`), precedence, parentheses,
right-associativity of `^`, floating-point precision (`0.1 + 0.2 = 0.3`),
chained and mixed operations, large numbers, and negative results.

### unary
Prefix `+` and `-`, double negation, unary operators inside expressions, unary
negation applied to built-ins, and the postfix factorial including `0!`,
non-integer and negative inputs, and overflow past `170!`.

### constants
`pi`, `e`, `inf`, `true`, `false`, `tau` and `phi`, implicit multiplication
(`2pi`), scientific notation against Euler's number (`9e` vs `9e2`), implicit
function application (`sin pi`), and `inf` arithmetic.

### variables
Assignment, reassignment, several variables in one expression, `ans` across
calls, `ans` chaining, and the errors for an uninitialised `ans` or an unknown
name.

### builtins
Every built-in function, including `clamp`, `gcd` and `lcm`. Also identities
and round-trips: the Pythagorean identity, `asin(sin(x))`, `ln(exp(x))`, and
composed calls.

### user_functions
Definition and invocation, several parameters, redefinition, nested calls,
arity mismatches, illegal parameters, implicit multiplication with a user
function, composition, and what may not be redefined (`ans`, constants and
built-ins).

### scoping
What a name in a body refers to: a parameter shadowing a global, a global read
at call time, a global defined *after* the function that reads it, a caller's
parameters deliberately out of reach, parameter order, and a parameter used as
a multiplicand or a dot-call receiver.

### caching
Memoisation, and everything that has to invalidate it: redefining a function,
changing a global that a body reads, and an answer a body depends on. Also
what must *not* discard it, since clearing too eagerly would make memoisation
pointless.

### statements
Several statements in one input: they run in order and the last one answers,
definitions carry forward, empty statements are skipped, and the first error
stops the rest while keeping what ran. Errors are checked to point into the
whole input, on the right line of a multi-line one, while a frame inside a call
still points into the definition. Also comments: one runs to the end of its
line, a separator inside one separates nothing, and a statement that is only a
comment is skipped.

### logic
Equality and comparison, boolean operators with `true`/`false` and numeric
truthiness, precedence between arithmetic, comparison and logic, and the
conditional operator including nesting and expressions in either branch.

### ufcs
Method-style calls on literals and user functions, no-argument and chained
calls, and the lexing this has to stay clear of: a decimal point against a
method dot (`10.5.max(5)`), an exponent followed by a call (`9e2.max(5)`), and
the malformed numbers that greedy consumption used to mask (`10.`, `9e9e9`).

### errors
Inputs that have to be rejected, asserted behaviourally: division by zero,
`0^negative`, `recip(0)`, mismatched parentheses, empty input, invalid tokens,
missing operators, `tan` at an asymptote, `sqrt(-1)`, `ln(-1)`, and calling
something undefined.

### diagnostics
What an error *reports*, rather than that one happened. The stage and the kind
for every failure the pipeline can produce, so a regression cannot quietly
change stage; and the rendered output, since a span that drifts is invisible
to a kind assertion — the carets under a call, a note beside them, and a frame
per call for a failure raised inside a function body.

### limits
The shapes that used to exhaust the native stack: a long flat chain, deep
parentheses, a deep body at depth, long runs of `+` and `-`, and runaway
recursion. A regression here does not fail an assertion, it aborts the test
runner, which is the only way to catch what these guard against.

### repl
The driver rather than an expression: what a definition returns, the forms
`help` accepts, and an answer surviving a failed line.
