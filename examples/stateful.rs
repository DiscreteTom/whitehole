//! This example demonstrates how to use the stateful parser to parse JavaScript template strings.

use in_str::in_str;
use whitehole::{
  action::Action,
  combinator::{eat, next},
  parser::Parser,
};

// define a custom state to track the nested level of JavaScript template strings
pub struct MyState {
  pub nested: usize,
}

// helper functions to manipulate the state and for better type inference.
// you can also make them methods of `MyState`.
fn inc(state: &mut MyState) {
  state.nested += 1;
}
fn dec(state: &mut MyState) {
  state.nested -= 1;
}
fn nested(state: &mut MyState) -> bool {
  state.nested != 0
}

pub fn build_lexer(s: &str) -> Parser<impl Action<str, MyState>, &str, MyState> {
  let body_optional = || {
    let escape = {
      let simple = next(in_str!("0'\"\\nrvtbf\u{000a}\u{000d}\u{2028}\u{2029}"));
      let hex = eat('x') + next(|c| c.is_ascii_hexdigit()) * 2;
      let unicode = eat('u') + next(|c| c.is_ascii_hexdigit()) * 4;
      let code_point = eat('u') + '{' + next(|c| c.is_ascii_hexdigit()) * (1..) + '}';
      eat('\\') + (simple | hex | unicode | code_point | '$')
    };

    let non_close = next(|c| c != '`' && c != '$') * (1..);

    let non_close_dollar = eat("$") + next(|c| c != '{');

    escape | non_close | non_close_dollar
  } * (..);

  // match "`..`" or "`..${"
  let whole_or_left = {
    let whole_end = eat('`');
    // if the template string is not closed, we need to increment the nested level
    let left_end = eat("${").then(|mut ctx| inc(ctx.state()));
    eat('`') + body_optional() + (whole_end | left_end)
  };

  // match "}..${" or "}..`"
  let middle_or_right = {
    let middle_end = eat("${");
    // if the template string is closed, we need to decrement the nested level
    let right_end = eat('`').then(|mut ctx| dec(ctx.state()));
    eat('}') + body_optional() + (right_end | middle_end)
  }
  // if not in a template string, the "}" is a normal character instead of part of a template string,
  // this action shouldn't be executed
  .prevent(|input| !nested(input.state));

  // other characters that are not part of a template string
  let others = {
    // when not in a template string, all non-"`" characters are normal characters
    let outside = (next(|c| c != '`') * (1..)).when(|input| !nested(input.state));
    // when in a template string, besides "`", we also need to check for "}" to handle middle_or_right
    let inside = (next(|c| c != '}' && c != '`') * (1..)).when(|input| nested(input.state));
    outside | inside
  };

  Parser::builder()
    .state(MyState { nested: 0 })
    .entry(others | whole_or_left | middle_or_right)
    .build(s)
}

fn main() {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_js_template_str_lexer() {
    let mut lexer = build_lexer("`begin${ `${ `123` }` }end`");
    while lexer.next().is_some() {}
    assert_eq!(lexer.instant().rest(), "");

    let mut lexer = build_lexer("`begin${ 123 }middle${ 456 }end`");
    while lexer.next().is_some() {}
    assert_eq!(lexer.instant().rest(), "");
  }
}
