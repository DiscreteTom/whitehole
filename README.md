# whitehole

![license](https://img.shields.io/github/license/DiscreteTom/whitehole?style=flat-square)
[![Crates.io Version](https://img.shields.io/crates/v/whitehole?style=flat-square)](https://crates.io/crates/whitehole)
[![docs.rs](https://img.shields.io/docsrs/whitehole?style=flat-square)](https://docs.rs/whitehole/)
[![Codecov](https://img.shields.io/codecov/c/github/DiscreteTom/whitehole?style=flat-square)](https://codecov.io/gh/DiscreteTom/whitehole)

A simple, fast, intuitive parser combinator framework for Rust.

## Features

- Simple: only a handful of combinators to remember: `eat`, `take`, `next`, `till`, `wrap`, `recur`.
- Operator overloading: use `+`, `|`, `!` to compose combinators, use `*` to repeat a combinator.
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
    .select(|accept, _| u8::from_str_radix(accept.content(), 16).unwrap())
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

## How to Debug

### With Logging

The easiest way is to apply `.log(name)` to any combinator you need to inspect.

<details>

<summary>
Example
</summary>

```rust
use whitehole::{
  combinator::{eat, next},
  parser::Parser,
};

let double_hex = || {
  (next(|c| c.is_ascii_hexdigit()).log("hex") * 2)
    .log("double_hex")
    .select(|accept, _| u8::from_str_radix(accept.content(), 16).unwrap())
    .tuple()
};

let entry =
  (eat('#').log("hash") + double_hex().log("R") + double_hex().log("G") + double_hex().log("B"))
    .log("entry");

let mut parser = Parser::builder().entry(entry).build("#FFA500");
parser.next().unwrap();
```

Output:

```text
(entry) input: "#FFA500"
| (hash) input: "#FFA500"
| (hash) output: Some("#")
| (R) input: "FFA500"
| | (double_hex) input: "FFA500"
| | | (hex) input: "FFA500"
| | | (hex) output: Some("F")
| | | (hex) input: "FA500"
| | | (hex) output: Some("F")
| | (double_hex) output: Some("FF")
| (R) output: Some("FF")
| (G) input: "A500"
| | (double_hex) input: "A500"
| | | (hex) input: "A500"
| | | (hex) output: Some("A")
| | | (hex) input: "500"
| | | (hex) output: Some("5")
| | (double_hex) output: Some("A5")
| (G) output: Some("A5")
| (B) input: "00"
| | (double_hex) input: "00"
| | | (hex) input: "00"
| | | (hex) output: Some("0")
| | | (hex) input: "0"
| | | (hex) output: Some("0")
| | (double_hex) output: Some("00")
| (B) output: Some("00")
(entry) output: Some("#FFA500")
```

</details>

If you need to inspect your custom state and heap, you can use combinator decorators or write your own combinator extensions to achieve this.

### With Breakpoints

Because of the high level abstraction, it's hard to set breakpoints to combinators.

One workaround is to use `wrap` to wrap your combinator in a closure or function and manually call `Action::exec`.

<details>

<summary>
Example
</summary>

```rust
use whitehole::{
  combinator::{eat, next},
  parser::Parser,
};

let double_hex = || {
  (next(|c| c.is_ascii_hexdigit()) * 2)
    .select(|accept, _| u8::from_str_radix(accept.content(), 16).unwrap())
    .tuple()
};
// wrap the original combinator
let double_hex = || {
  use whitehole::{action::Action, combinator::wrap};
  let c = double_hex();
  wrap(move |instant, ctx| {
    // set a breakpoint here
    c.exec(instant, ctx)
  })
};

let entry = eat('#') + double_hex() + double_hex() + double_hex();

let mut parser = Parser::builder().entry(entry).build("#FFA500");
parser.next().unwrap();
```

</details>

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
