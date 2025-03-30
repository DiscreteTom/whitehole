# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add `Input` back.
- Add `Accepted::state` and `Accepted::heap`.
- Add `!` operator overloading for `Combinator`.
- Add `Contextual` and `contextual!`.
- Add `new` for provided combinators.
- Add combinator functions and structs for all bytes combinators.
- Add `Accepted::new_unchecked`.

### Changed

- `Text`, `State` and `Heap` are now associated types of `Action` instead of generic params.
- `Action::exec` now only takes `Input` as the parameter.
- All provided combinators will associate `State` and `Heap` as `()` instead of generic, and associate a `Text` type.
- Contextual combinators can't take non-static reference as the heap.
- Combinator decorators with closure as the parameter type will accept `Input` or `Accepted` as the only parameter.
- `NoSep` is now generic.
- `wrap` and `wrap_unchecked`'s return type will be wrapped in `Contextual`.
- `Parser` now only takes `'text` and the entry action as generic params.

### Removed

- Remove `Context`. Use `Input` instead.
- Remove the implementation of `PartialEq` and `Eq` for `Snapshot` and `Instant` to prevent unintended string comparison.

## [0.7.0] - 2025-02-16

### Added

- Add `Digest::as_bytes`, `Digest::get` and `Digest::get_unchecked`.
- Add `Instant::accept_unchecked` and `Instant::accept` to replace `Input::digest_unchecked` and `Input::digest`.
- Add `Instant::to_digested_unchecked` to replace `Input::shift_unchecked`.
- Add `Context` to provide `&mut State` and `&mut Heap`.
- Add `Output::as_ref`.
- Derive `Clone` for `Accepted`.
- Make `Accepted::end` and `Accepted::range` const.

### Changed

- `Digest` is now implemented by `str` and `[u8]` instead of `&str` and `&[u8]`.
- `Action` now takes `Instant` and `Context` as params.
- Rename `AcceptedContext` to `Accepted`.
- `Accepted` now contains `Instant` and `Output`. Simplify its generic params.
- Rename `Accepted::rest` to `Accepted::after`.
- `Log` now can only take `&str` as the name.

### Fixed

- Fix `till` in bytes context when the provided byte sequence is empty.

### Removed

- Remove `Digest::len` and `Digest::is_empty`, use `Digest::as_bytes` instead.
- Remove `Digest::digest_unchecked` and `Digest::span_unchecked`, use `Digest::get_unchecked` instead.
- Remove `Input`, use `Instant` and `Context` instead.
- Remove `Accepted::split`.

## [0.6.1] - 2025-02-13

### Added

- Make `AcceptedContext::input` and `AcceptedContext::output` available in `const` context.
- Add `Combinator::log` to debug combinators.

## [0.6.0] - 2025-01-31

### Added

- Add `Combinator::bind_with`.
- Implement `Iterator` for `Parser`.

### Removed

- Remove `Combinator::bind_default`, use `Combinator::bind_with` instead.
- Remove `Parser::parse`, use `Parser::next` instead.

## [0.5.0] - 2025-01-28

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

[unreleased]: https://github.com/DiscreteTom/whitehole/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.7.0
[0.6.1]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.6.1
[0.6.0]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.6.0
[0.5.0]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.5.0
[0.4.0]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.4.0
[0.3.0]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.3.0
[0.2.0]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.2.0
[0.1.0]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.1.0
[0.0.1]: https://github.com/DiscreteTom/whitehole/releases/tag/v0.0.1
