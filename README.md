# calc

Calc is a mathematical evaluation engine and REPL implemented in Rust. It uses a Pratt parser for operator precedence and a two-stage AST pipeline to support dynamic variables, user-defined functions, and recursive expressions.

## Installation

```bash
git clone https://github.com/S-Ebad/calc
cd calc
cargo run --release
```

## Usage
```
> 2 + 3 * 4
= 14

> x = 5
= 5

> x^2 + 1
= 26

> f(n) = n^2 + 1

> f(x)
= 26

> sin(pi / 2)
= 1

> fib(x) = x > 1 ? fib(x - 1) + fib(x - 2) : x

> fib(6)
= 8
```

## Docs

- [Features](features.md): supported operators, functions, constants, variables, and syntax
- [Tests](tests.md): test suite overview

## Pipeline

Input goes through four stages:
* **Lexing** — raw string to tokens
* **Parsing** — tokens to unresolved AST (`RawExpr`) via a Pratt parser
* **Resolving** — each node of a `RawExpr` is looked up against the current variables and functions: identifiers, constants, and calls are validated, one node at a time
* **Evaluation** — the `RawExpr` is reduced to an `f64` in place, resolving each node as it is reached. A typed expression is checked in full first; a function body is evaluated where it is stored, so a call copies nothing and only looks at the branch it takes
