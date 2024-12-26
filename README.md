# whitehole

![license](https://img.shields.io/github/license/DiscreteTom/whitehole?style=flat-square)
[![Crates.io Version](https://img.shields.io/crates/v/whitehole?style=flat-square)](https://crates.io/crates/whitehole)
[![docs.rs](https://img.shields.io/docsrs/whitehole?style=flat-square)](https://docs.rs/whitehole/)

<!-- TODO: coverage -->

A simple, fast, intuitive parser combinator framework for Rust.

## Features

- Simple: only a handful of combinators to remember: `eat`, `eater`, `next`, `till`, `wrap`.
- Operator overloading: use `+` and `|` to compose combinators, use `*` to repeat a combinator.
- Zero heap allocation: this framework only uses stack memory.
- Re-usable heap memory: if you need allocation, never clone or reallocate it.
- Stateful-able: control the parsing flow with an optional custom state.
- Safe by default, with `unsafe` variants for performance.

## Installation

```bash
cargo add whitehole
```

## [Examples](./examples)

TODO: put a simple example here.

## [Documentation](https://docs.rs/whitehole/)

## Credits

This project is inspired by:

- [nom](https://github.com/rust-bakery/nom)
- [combine](https://github.com/Marwes/combine)
- [retsac](https://github.com/DiscreteTom/retsac)
- [tree-sitter](https://github.com/tree-sitter/tree-sitter)

## [CHANGELOG](./CHANGELOG.md)
