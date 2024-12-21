# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add `Combinator!(@T)` and `Combinator!(Kind, @T)` syntax.
- Add `Combinator::range` to include the byte range of digested text in the output.
- Add `Combinator::when` as the opposite of `Combinator::prevent`.

### Changed

- Rename `Node` to `WithRange`, rename its field `kind` to `data`.
- Make generic param `State` and `Heap` of the `Parse` trait become associated type.
- Rename `Parse::Kind` to `Parse::Value`, rename `Output::kind` to `Output::value`.
- Plain closures are no longer supported as `Parse` trait implementations. Use `wrap` instead to create a `Combinator` from a closure.
- Simplify generic params of `Combinator` and `Parser`.
- `Parser::parse` and `Parser::peek` will return `parse::Output` instead of `WithRange`.

### Removed

- Remove `combinator::Builder`.
- Remove `Combinator!(_, State, Heap)` syntax.

## [0.0.1] - 2024-11-24

### Added

- The `Parse` trait and the related `Input` and `Output` struct.
- The `Combinator` struct and the same name macro.
- Built-in combinators: `eat`, `next`, `till`, `eat_unchecked`, `eater`, `eater_unchecked`, `wrap`.
- Overloaded operators: `|`, `+` and `*` for `Combinator`.
- The basic `Parser`, with `Builder`, `Instant` and `Snapshot` support.

[unreleased]: https://github.com/DiscreteTom/whitehole/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.0.1
