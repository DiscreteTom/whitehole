//! ## For Developers
//!
//! Here is the recommended order of reading the source code:
//!
//! 1. [`self::body`]
//! 2. [`self::options`]
//! 3. [`self::escape`]
//! 4. [`self`]

mod body;
mod error;
mod escape;
mod options;
mod value;

use std::collections::HashSet;

pub use body::*;
pub use error::*;
pub use escape::*;
pub use options::*;
pub use value::*;

use super::{Accumulator, StringList};
use crate::lexer::{
  action::{simple_with_data, Action},
  token::MockTokenKind,
};

// TODO: comments
pub fn string_literal<
  ActionState,
  ErrorType,
  Value: PartialStringBodyValue + 'static,
  CustomError: 'static,
  BodyAcc: Accumulator<PartialStringBody<Value, CustomError>> + Clone,
>(
  open: impl Into<StringList>,
  options: StringBodyOptions<Value, CustomError, BodyAcc>,
) -> Action<MockTokenKind<BodyAcc>, ActionState, ErrorType> {
  let open: Vec<String> = open.into().0;
  let head: HashSet<_> = open
    .iter()
    .map(|s| {
      s.chars()
        .next()
        .expect("string literal's open quote should not be empty")
    })
    .collect();

  simple_with_data(move |input| {
    for prefix in &open {
      if input.rest().starts_with(prefix) {
        let (body_len, data) = string_body(&input.rest()[prefix.len()..], &options);
        return Some((prefix.len() + body_len, data));
      }
    }
    // no prefix matched
    return None;
  })
  .unchecked_head_in(head)
}
