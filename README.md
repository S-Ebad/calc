# calc

Calc is a high-performance mathematical evaluation engine and REPL implemented in Rust. It leverages a Pratt Parser for complex operator precedence and a recursive-descent resolver to support dynamic variables, user-defined functions, and sophisticated constant inlining

## Installation

```bash
git clone https://github.com/S-Ebad/calc
cd calc
cargo build --release
./target/release/calc
```

Or just run directly:

```bash
cargo run
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
```

## Docs

- [Features](features.md): supported operators, functions, constants, variables, and syntax
- [Tests](tests.md): test suite overview

## Pipeline

Input goes through four stages: 
* **lexing** (raw string -> tokens)
* **parsing** (tokens -> AST via a Pratt parser)
* **resolving** (identifiers, constants, and user functions are substituted into the tree)
*  **evaluation** (the tree is walked and reduced to a number).
