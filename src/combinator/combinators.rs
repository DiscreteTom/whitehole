#[macro_export]
macro_rules! combinators {
  ($(let $name:ident = || $body:expr;)+) => {
    $(
      pub fn $name() -> $crate::combinator::Combinator<impl $crate::action::Action<Text = str, State = (), Heap = (), Value = ()>> {
        $body
      }
    )+
  };
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::action::Action;
  use crate::combinator::{eat, next};
  use in_str::in_str;

  #[test]
  fn test_combinators() {
    combinators! {
      let simple = || eat('a');
      let complex = || {
        let a = eat('a');
        a + 'b'
      };
    }
  }
}
