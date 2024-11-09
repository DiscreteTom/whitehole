use super::eater_unchecked;
use crate::{
  combinator::Combinator,
  utils::lookup::{
    char::{SparseCharLookupTable, SparseCharLookupTableBuilder},
    Lookup,
  },
};
use std::{collections::HashSet, ops::RangeBounds};

/// Match chars by the condition greedily.
/// Reject if no char is matched.
///
/// # Examples
/// ```
/// # use whitehole::combinator::{Combinator, chars};
/// // match all ascii digits greedily
/// let _: Combinator<_> = chars(|ch| ch.is_ascii_digit());
/// ```
pub fn chars<'a, State, Heap>(
  condition: impl Fn(char) -> bool + 'a,
) -> Combinator<'a, (), State, Heap> {
  eater_unchecked(move |input| {
    let mut digested = 0;
    for c in input.rest().chars() {
      if !condition(c) {
        break;
      }
      digested += c.len_utf8();
    }
    digested
  })
}

/// Match chars in the range greedily.
/// Reject if no char is matched.
///
/// # Examples
/// ```
/// # use whitehole::combinator::{Combinator, chars_in_range};
/// // match all ascii digits greedily with RangeInclusive
/// let _: Combinator<_> = chars_in_range('0'..='9');
/// ```
pub fn chars_in_range<'a, State, Heap>(
  range: impl RangeBounds<char> + 'a,
) -> Combinator<'a, (), State, Heap> {
  chars(move |c| range.contains(&c))
}

/// Match chars in the set greedily.
/// Reject if no char is matched.
///
/// This will use an optimized lookup table for performance.
/// # Examples
/// ```
/// # use whitehole::combinator::{Combinator, chars_in_set};
/// // match 'a' or 's' or 'd' greedily
/// let _: Combinator<_> = chars_in_set(['a', 's', 'd']);
/// ```
pub fn chars_in_set<'a, State, Heap>(
  set: impl Into<HashSet<char>>,
) -> Combinator<'a, (), State, Heap> {
  let set = set.into();
  let table: SparseCharLookupTable<()> =
    SparseCharLookupTableBuilder::new(set.iter().copied().collect()).build();
  chars(move |ch| table.get(ch as usize).is_some())
}

/// Match chars in the string greedily.
/// Reject if no char is matched.
///
/// This will use an optimized lookup table for performance.
/// # Examples
/// ```
/// # use whitehole::combinator::{Combinator, chars_in_str};
/// // match 'a' or 's' or 'd' greedily
/// let _: Combinator<_> = chars_in_str("asd");
/// ```
pub fn chars_in_str<'a, State, Heap>(s: &str) -> Combinator<'a, (), State, Heap> {
  chars_in_set(s.chars().collect::<HashSet<_>>())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::Input;

  #[test]
  fn combinator_chars() {
    // normal
    assert_eq!(
      chars(|_| true)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(chars(|_| false)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_chars_in_range() {
    // normal
    assert_eq!(
      chars_in_range('0'..='9')
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(chars_in_range('a'..='z')
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
    // other range
    let _: Combinator<_> = chars_in_range('\x00'..'\x05'); // Range
    let _: Combinator<_> = chars_in_range('a'..); // RangeFrom
    let _: Combinator<_> = chars_in_range(..'z'); // RangeTo
    let _: Combinator<_> = chars_in_range(..); // RangeFull
    let _: Combinator<_> = chars_in_range('a'..='z'); // RangeInclusive
    let _: Combinator<_> = chars_in_range(..='z'); // RangeToInclusive
  }

  #[test]
  fn combinator_chars_in_set() {
    // normal
    assert_eq!(
      chars_in_set(['1', '2', '3'])
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(chars_in_set(['a', 'b', 'c'])
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_chars_in_str() {
    // normal
    assert_eq!(
      chars_in_str("123")
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(chars_in_str("abc")
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }
}
