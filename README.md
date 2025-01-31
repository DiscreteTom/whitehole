# whitehole

![license](https://img.shields.io/github/license/DiscreteTom/whitehole?style=flat-square)
[![Crates.io Version](https://img.shields.io/crates/v/whitehole?style=flat-square)](https://crates.io/crates/whitehole)
[![docs.rs](https://img.shields.io/docsrs/whitehole?style=flat-square)](https://docs.rs/whitehole/)
[![Codecov](https://img.shields.io/codecov/c/github/DiscreteTom/whitehole?style=flat-square)](https://codecov.io/gh/DiscreteTom/whitehole)

A simple, fast, intuitive parser combinator framework for Rust.

## Features

- Simple: only a handful of combinators to remember: `eat`, `take`, `next`, `till`, `wrap`, `recur`.
- Operator overloading: use `+` and `|` to compose combinators, use `*` to repeat a combinator.
- Almost zero heap allocation: this framework only uses stack memory, except `recur` which uses some pointers for recursion.
- Re-usable heap memory: store accumulated values in a parser-managed heap, instead of re-allocation for each iteration.
- Stateful-able: control the parsing flow with an optional custom state.
- Safe by default, with `unsafe` variants for performance.
- Provide both string (`&str`) and bytes (`&[u8]`) support.

## Installation

```bash
cargo add whitehole
```

## Examples

See the [examples](./examples) directory for more examples.

Here is a simple example to parse [hexadecimal color codes](./examples/hex_color.rs):

```rust
use whitehole::{
  combinator::{eat, next},
  parser::Parser,
};

let double_hex = || {
  // Repeat a combinator with `*`.
  (next(|c| c.is_ascii_hexdigit()) * 2)
    // Convert the matched content to `u8`.
    .select(|ctx| u8::from_str_radix(ctx.content(), 16).unwrap())
    // Wrap `u8` to `(u8,)`, this is required by `+` below.
    .tuple()
};

// Concat multiple combinators with `+`.
// Tuple values will be concatenated into a single tuple.
// Here `() + (u8,) + (u8,) + (u8,)` will be `(u8, u8, u8)`.
let entry = eat('#') + double_hex() + double_hex() + double_hex();

let mut parser = Parser::builder().entry(entry).build("#FFA500");
let output = parser.next().unwrap();
assert_eq!(output.digested, 7);
assert_eq!(output.value, (0xFF, 0xA5, 0x00));
```

## [Documentation](https://docs.rs/whitehole/)

## [Benchmarks](https://github.com/DiscreteTom/whitehole-bench)

## Related

- [`in_str`](https://github.com/DiscreteTom/in_str/): a procedural macro to generate a closure that checks if a character is in the provided literal string.

## Credits

This project is inspired by:

- [nom](https://github.com/rust-bakery/nom)
- [combine](https://github.com/Marwes/combine)
- [tree-sitter](https://github.com/tree-sitter/tree-sitter)
- [retsac](https://github.com/DiscreteTom/retsac)

## [CHANGELOG](./CHANGELOG.md)
