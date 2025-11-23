use crate::common::{number, string, whitespaces};
use in_str::in_str;
use whitehole::{
  action::Action,
  combinator::{next, Combinator},
  entry,
};

entry! {
  let lexer_entry = || {
    let boundary = next(in_str!("[]{}:,"));
    whitespaces() | boundary | number() | string() | "true" | "false" | "null"
  };
}
