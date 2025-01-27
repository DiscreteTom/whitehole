# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add trait `Digest`.
- Add `Input::shift_unchecked`.
- Add `AcceptedContext::output`.
- Add `combinator::bytes`.
- Add `Eat<u8>`, `Eat<&[u8]>`, `Eat<&[u8;N]>`, `Eat<Vec<u8>>`.
- Add `take`/`Take`.
- Add `Till<u8>`, `Till<&[u8]>`, `Till<&[u8;N]>`, `Till<Vec<u8>>`.
- Add `recur`/`recur_unchecked`/`Recur`/`RecurUnchecked`/`RecurSetter`.
- Add `NoSep`.
- Add `Parser::builder`.

### Changed

- `Instant` now take a generic param `TextRef` instead of a lifetime param.
- `Input` now take an additional generic param `TextRef` instead of a lifetime param.
- The `rest` of `Input::instant` is no longer guaranteed to be non-empty.
- `Input::new` will return `Input` directly instead of an `Option`.
- `Action` will now take an additional generic param `Text`.
- `AcceptedContext::output` will always be a struct instead of a reference.
- `eat` will no longer accept `usize`, use `take` instead.
- Rewrite `Mul`.
- `Combinator::fold` now only exists for `Combinator<Mul>` and will return a `Combinator<Mul>`.
- `Combinator::fold`'s `fold` now take `acc` as the first param and `value` as the second param.
- `Parser` now take an additional generic param `TextRef` instead of a lifetime param.
- `Snapshot` now take an additional generic param `TextRef` instead of a lifetime param.

### Fixed

- `Combinator::optional` now will work properly with empty rest string.

### Removed

- Remove `Input::new_unchecked` and `Input::next`.
- Remove implementation of `Action` for `&mut Action`.
- Remove `Eat<&String>`. Use `Eat<&str>` and `eat(s.as_str())` instead.
- Remove `eat_unchecked`. Use `wrap` instead.
- Remove `Sep`, `Fold` and `InlineFold`.

## [0.4.0] - 2025-01-10

### Added

- Add `Combinator::fold`.

### Changed

- `Combinator::sep` now should be used after `mul` and will return a combinator.

### Removed

- Remove `combinator * (repeat, init, fold)` syntax for inline fold. Use `Combinator::fold` instead for better type inference.

## [0.3.0] - 2025-01-05

### Added

- Add `Input::validate`.
- Implement `Action` for `&dyn Action`, `&mut dyn Action`, `Box<dyn Action>` and `Rc<dyn Action>`.
- Add `Combinator::action`.
- Add `WrapUnchecked`, `Wrap`, `Next`, `EatUnchecked`.
- `eat`, `add` and `bitor` can accept `&String`.
- Add `When`, `Prevent`, `Reject`, `Optional`, `Boundary`, `Prepare`, `Then`, `Catch`, `Finally`, `Map`, `Tuple`, `Bind`, `BindDefault`, `Select`, `Range`, `Pop`.

### Changed

- Make `Action::State` and `Action::Heap` become generic params.
- Provided combinator constructors will return concrete types instead of `impl` types to retain trait information.
- `Eat` and `Till` are now structs instead of traits.
- Combinator repetition with `Fold` will be implemented separately to improve performance and simplify type signature.

### Removed

- Remove `Input::shift_unchecked` and `Input::shift`.
- Remove `C!` macro.
- Remove `eater` and `eater_unchecked`. Use `wrap` instead.
- Remove `EatChar`, `EatStr`, `EatString`, `EatUsize`. Use `Eat<T>` instead.

## [0.2.0] - 2025-01-01

### Added

- Add `Input::instant`.
- Implement `Action` for `&Action` and `&mut Action`.
- Add `Combinator::pop`.

### Changed

- `Input::new` and `Input::new_unchecked` will require `Instant` instead of `start` and `rest`.
- `Fold` requires 2 generic params: `State` and `Heap`.
- `Fold::fold` and inline fold will provide `input`.
- Move `parser::Instant` to `instant::Instant`.

### Removed

- Remove `Input::start` and `Input::rest`, use `Input::instant` instead.

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

[unreleased]: https://github.com/DiscreteTom/whitehole/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.4.0
[0.3.0]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.3.0
[0.2.0]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.2.0
[0.1.0]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.1.0
[0.0.1]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.0.1
