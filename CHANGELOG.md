# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Implement `Action` for `&Action` and `&mut Action`.

### Changed

- `Fold` requires 2 generic params: `State` and `Heap`.
- `Fold::fold` and inline fold will provide `input`.
- Move `parser::Instant` to `instant::Instant`.

## [0.1.0] - 2024-12-29

### Added

- Add `Input::new_unchecked`.
- Add `Input::reborrow`.
- Add `Input::shift` and `Input::shift_unchecked`.
- Add `Output::digested`.
- Implement `Action` for `Box<Action>`, `Rc<Action>`.
- Add `C!(@T)` and `C!(Kind, @T)` syntax.
- Add `AcceptedContext::input`, `AcceptedContext::start`, `AcceptedContext::state`, `AcceptedContext::heap`, `AcceptedContext::rest`, `AcceptedContext::take`, `AcceptedContext::split`.
- Add `Combinator::range` to include the byte range of digested text in the output.
- Add `Combinator::when` as the opposite of `Combinator::prevent`.
- Add `Combinator::finally` to modify `Input` after execution.
- Add `wrap_unchecked`.
- Implement `From<char>`, `From<String>`, `From<usize>` and `From<&str>` for `Combinator`.
- Implement `Eat` for `EatChar`, `EatStr`, `EatString`, `EatUsize`.
- Add `Combinator::sep`.
- All provided combinators are now `const`.
- Add `Instant::digest_unchecked`.
- Add `Parser::digest_unchecked`, `Parser::digest_with_unchecked`.
- `Parser::reload_with` can accept `None` as the new state.

### Changed

- Rename `Node` to `WithRange`, rename its field `kind` to `data`.
- Rename `Parse` to `Action`, rename its method `parse` to `exec`.
- `Action::exec` will consume the `Input`.
- Mark `Action` unsafe.
- Rewrite `Output`, remove it's lifetime param.
- Make generic param `State` and `Heap` of the `Parse` trait become associated type.
- Rename `Action::Kind` to `Action::Value`, rename `Output::kind` to `Output::value`.
- Rename `Combinator!` to `C!`.
- Rename `Combinator::rollback` to `Combinator::catch`.
- Make fields of `AcceptedContext` private.
- `wrap` will check if the `Output::digested` is valid. Use `wrap_unchecked` for the unchecked version.
- `EatUsize` will eat by chars instead of bytes.
- Rewrite `Till` trait to make it safe.
- Combinator repetition with `Fold` will be implemented by using inline fold.
- Make `Repeat::validate` unsafe.
- Simplify generic params of `Combinator` and `Parser`.
- `Parser::parse` and `Parser::peek` will return `parse::Output` instead of `WithRange`.

### Removed

- Remove `Input::reload`, use `Input::shift` instead.
- Remove `Output::rest`.
- Remove `Action` trait implementation for plain closures. Use `wrap` instead to create a `Combinator` from a closure.
- Remove `combinator::Builder`.
- Remove `C!(_, State, Heap)` syntax.
- Remove `combinator * (repeat, sep)` syntax.
- Remove `Instant::update`.

## [0.0.1] - 2024-11-24

### Added

- The `Parse` trait and the related `Input` and `Output` struct.
- The `Combinator` struct and the same name macro.
- Built-in combinators: `eat`, `next`, `till`, `eat_unchecked`, `eater`, `eater_unchecked`, `wrap`.
- Overloaded operators: `|`, `+` and `*` for `Combinator`.
- The basic `Parser`, with `Builder`, `Instant` and `Snapshot` support.

[unreleased]: https://github.com/DiscreteTom/whitehole/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.1.0
[0.0.1]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.0.1
