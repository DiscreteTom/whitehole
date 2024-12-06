# CHANGELOG

## v0.0.2

- **_Breaking Change_**: make generic param `State` and `Heap` of the `Parse` trait become associated type.
- **_Breaking Change_**: plain closures are no longer supported as `Parse` trait implementations.
- **_Breaking Change_**: remove `combinator::Builder`.
- **_Breaking Change_**: remove `Combinator!(_, State, Heap)` syntax, add `Combinator!(@T)` and `Combinator!(Kind, @T)` syntax.

## v0.0.1

The initial release.
