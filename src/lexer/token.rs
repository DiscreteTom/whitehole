use crate::kind::KindIdBinding;

pub type Range = std::ops::Range<usize>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token<Kind> {
  /// The token kind id and value.
  pub binding: KindIdBinding<Kind>,
  /// The byte range of the token in the input text.
  /// This can be used to index the input text.
  /// # Example
  /// ```
  /// # use whitehole::lexer::token::{Token, MockKind};
  /// let token = Token {
  ///   binding: MockKind::new(()).into(),
  ///   range: 0..5,
  /// };
  /// // index a string with the range
  /// assert_eq!(&"0123456"[token.range], "01234");
  pub range: Range,
  // we don't store `token.content` here (as a `&str`).
  // when lexing users can get the content in the lexing context,
  // parse its value if needed and store the result data in `self.binding.kind`.
  // `token.content` may only be used less than once, and can be calculated from `token.range`.
  // users can calculate and cache it by themselves, we don't do unnecessary work.
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::kind::{DefaultSubKind, KindId, MockKind, SubKind};
  use whitehole_macros::_kind;

  #[_kind]
  #[derive(Debug, Clone, Default, PartialEq, Eq)]
  pub enum MyKind {
    #[default]
    Unit,
    Unnamed(i32),
    Named {
      name: i32,
    },
  }

  #[test]
  fn test_token() {
    let token = Token {
      binding: MockKind::new(()).into(),
      range: 0..5, // ensure we can create the range with the range syntax
    };

    // ensure the range can be used to index a string
    assert_eq!(&"0123456"[token.range], "01234");
  }

  #[test]
  fn kind_macro() {
    // generated structs
    let _ = Unit;
    let _ = Unnamed(42);
    let _ = Named { name: 42 };

    // unit variant is still unit variant instead of an unnamed variant
    let _ = MyKind::Unit;

    // other variants are transformed into unnamed variants
    let _ = MyKind::Unnamed(Unnamed(42));
    let _ = MyKind::Named(Named { name: 42 });

    // sub token kinds into token kind
    assert!(matches!(Unit.into(), MyKind::Unit));
    assert!(matches!(Unnamed(42).into(), MyKind::Unnamed(Unnamed(42))));
    assert!(matches!(
      Named { name: 42 }.into(),
      MyKind::Named(Named { name: 42 })
    ));

    // into token kind id binding
    let b: KindIdBinding<MyKind> = Unit.into();
    assert_eq!(b.id(), Unit::kind_id());
    assert_eq!(b.take(), MyKind::Unit);
    let b: KindIdBinding<MyKind> = Unnamed(42).into();
    assert_eq!(b.id(), Unnamed::kind_id());
    assert_eq!(b.take(), MyKind::Unnamed(Unnamed(42)));
    let b: KindIdBinding<MyKind> = Named { name: 42 }.into();
    assert_eq!(b.id(), Named::kind_id());
    assert_eq!(b.take(), MyKind::Named(Named { name: 42 }));

    // generated token kind id, as sub token kind.
    let v: Vec<KindId<MyKind>> = vec![Unit::kind_id(), Unnamed::kind_id(), Named::kind_id()];
    for (i, id) in v.iter().enumerate() {
      for (j, id2) in v.iter().enumerate() {
        if i == j {
          assert_eq!(id, id2);
        } else {
          assert_ne!(id, id2);
        }
      }
    }

    // sub token kind into token kind id
    assert_eq!(<Unit as Into<KindId<MyKind>>>::into(Unit), Unit::kind_id());
    assert_eq!(
      <Unnamed as Into<KindId<MyKind>>>::into(Unnamed(42)),
      Unnamed::kind_id()
    );
    assert_eq!(
      <Named as Into<KindId<MyKind>>>::into(Named { name: 42 }),
      Named::kind_id()
    );

    // attributes are inherited by generated structs, e.g. Clone
    let _ = Unit.clone();
    let _ = Unnamed(42).clone();
    let _ = Named { name: 42 }.clone();
    let _ = MyKind::Unit.clone();
    let _ = MyKind::Unnamed(Unnamed(42)).clone();
    let _ = MyKind::Named(Named { name: 42 }).clone();

    // default is working
    assert!(matches!(MyKind::default(), MyKind::Unit));
    assert_eq!(MyKind::default_kind_id(), Unit::kind_id());
  }
}
