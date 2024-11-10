use crate::combinator::{eater_unchecked, Combinator};

/// A util trait to make [`comment`] generic over different types.
///
/// Built-in implementations are provided for [`String`], `&str`, and [`char`].
pub trait CommentOpenQuote {
  /// Check if the input starts with this instance.
  fn is_prefix_of(&self, input: &str) -> bool;
  /// Get the byte length of the prefix.
  fn byte_len(&self) -> usize;
}

impl CommentOpenQuote for String {
  fn is_prefix_of(&self, input: &str) -> bool {
    input.starts_with(self)
  }
  fn byte_len(&self) -> usize {
    self.len()
  }
}

impl CommentOpenQuote for &str {
  fn is_prefix_of(&self, input: &str) -> bool {
    input.starts_with(self)
  }
  fn byte_len(&self) -> usize {
    self.len()
  }
}

impl CommentOpenQuote for char {
  fn is_prefix_of(&self, input: &str) -> bool {
    input.starts_with(*self)
  }
  fn byte_len(&self) -> usize {
    self.len_utf8()
  }
}

/// A util trait to make [`comment`] generic over different types.
///
/// Built-in implementations are provided for [`String`], `&str`, and [`char`].
pub trait CommentCloseQuote {
  /// Check if the input contains this instance.
  fn find_in(&self, input: &str) -> Option<usize>;
  /// Get the byte length of the prefix.
  fn byte_len(&self) -> usize;
}

impl CommentCloseQuote for String {
  fn find_in(&self, input: &str) -> Option<usize> {
    input.find(self)
  }
  fn byte_len(&self) -> usize {
    self.len()
  }
}

impl CommentCloseQuote for &str {
  fn find_in(&self, input: &str) -> Option<usize> {
    input.find(self)
  }
  fn byte_len(&self) -> usize {
    self.len()
  }
}

impl CommentCloseQuote for char {
  fn find_in(&self, input: &str) -> Option<usize> {
    input.find(*self)
  }
  fn byte_len(&self) -> usize {
    self.len_utf8()
  }
}

/// Match from the `open` to the `close`, including the `open` and `close`.
/// If the `close` is not found, accept all the rest.
/// # Examples
/// ```
/// # use whitehole::combinator::{Combinator, comment};
/// // single line comment
/// let _: Combinator<_> = comment("//", '\n');
/// let _: Combinator<_> = comment('#', '\n');
/// // multi line comment
/// let _: Combinator<_> = comment("/*", "*/");
/// let _: Combinator<_> = comment("<!--", "-->");
/// ```
pub fn comment<'a, State, Heap>(
  open: impl CommentOpenQuote + 'a,
  close: impl CommentCloseQuote + 'a,
) -> Combinator<'a, (), State, Heap> {
  eater_unchecked(move |input| {
    // reject if open mismatch
    if !open.is_prefix_of(input.rest()) {
      return 0;
    }

    close
      .find_in(&input.rest()[open.byte_len()..])
      // if match, return total length
      .map(|i| i + open.byte_len() + close.byte_len())
      // if the close is not found,
      // accept all rest as the comment
      .unwrap_or(input.rest().len())
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::Input;

  #[test]
  fn combinator_comment() {
    // normal
    assert_eq!(
      comment("//", '\n')
        .parse(&mut Input::new("//123\n123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(6)
    );
    // no open
    assert_eq!(
      comment("//", '\n')
        .parse(&mut Input::new("123\n", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    assert_eq!(
      comment("//", '\n')
        .parse(&mut Input::new("123//123\n", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    // no close
    assert_eq!(
      comment("//", "\n")
        .parse(&mut Input::new("//123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(5)
    );
    // multiline
    assert_eq!(
      comment("/*", "*/")
        .parse(&mut Input::new("/*123\n*/123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(8)
    );
    // multiline no open
    assert_eq!(
      comment("/*", "*/")
        .parse(&mut Input::new("123\n*/123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    assert_eq!(
      comment("/*", "*/")
        .parse(&mut Input::new("123/*123\n*/123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    // multiline no close
    assert_eq!(
      comment("/*", "*/")
        .parse(&mut Input::new("/*123\n", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(6)
    );
  }
}
