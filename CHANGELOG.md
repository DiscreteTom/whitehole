# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add `C!(@T)` and `C!(Kind, @T)` syntax.
- Add `Combinator::range` to include the byte range of digested text in the output.
- Add `Combinator::when` as the opposite of `Combinator::prevent`.
- Add `Combinator::finally` to modify `Input` after execution.
- Expose `Add::lhs`, `Add::rhs`, `BitOr::lhs` and `BitOr::rhs`.

### Changed

- Rename `Node` to `WithRange`, rename its field `kind` to `data`.
- Rename `Parse` to `Action`, rename its method `parse` to `exec`.
- Make generic param `State` and `Heap` of the `Parse` trait become associated type.
- Rename `Action::Kind` to `Action::Value`, rename `Output::kind` to `Output::value`.
- Rename `Combinator!` to `C!`.
- Rename `Combinator::rollback` to `Combinator::catch`.
- Combinator repetition with `Fold` will be implemented by using inline fold.
- Simplify generic params of `Combinator` and `Parser`.
- `Parser::parse` and `Parser::peek` will return `parse::Output` instead of `WithRange`.

### Removed

- Remove `Action` trait implementation for plain closures. Use `wrap` instead to create a `Combinator` from a closure.
- Remove `combinator::Builder`.
- Remove `C!(_, State, Heap)` syntax.
- Remove `combinator * (repeat, sep)` syntax.

## [0.0.1] - 2024-11-24

### Added

- The `Parse` trait and the related `Input` and `Output` struct.
- The `Combinator` struct and the same name macro.
- Built-in combinators: `eat`, `next`, `till`, `eat_unchecked`, `eater`, `eater_unchecked`, `wrap`.
- Overloaded operators: `|`, `+` and `*` for `Combinator`.
- The basic `Parser`, with `Builder`, `Instant` and `Snapshot` support.

[unreleased]: https://github.com/DiscreteTom/whitehole/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.0.1
