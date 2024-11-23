use in_str::in_str;
use whitehole::{
  combinator::{eat, next, AcceptedContext},
  parse::{Input, Parse},
  parser::{Builder, Parser},
};

pub struct MyState {
  pub nested: usize,
}

pub fn build_lexer(s: &str) -> Parser<impl Parse<MyState, Kind = ()>, MyState> {
  Builder::new()
    .state(MyState { nested: 0 })
    .entry(|_| {
      let escape = || {
        let simple = next(in_str!("0'\"\\nrvtbf\u{000a}\u{000d}\u{2028}\u{2029}"));
        let hex = eat('x') + next(|c| c.is_ascii_hexdigit()) * 2;
        let unicode = eat('u') + next(|c| c.is_ascii_hexdigit()) * 4;
        let code_point = eat('u') + '{' + next(|c| c.is_ascii_hexdigit()) * (1..) + '}';
        eat('\\') + (simple | hex | unicode | code_point | '$')
      };

      let body_optional =
        || (escape() | next(|c| c != '`' && c != '$') | (eat("$") + next(|c| c != '{'))) * (..);

      macro_rules! Input {
        () => {
          &mut Input<&mut MyState, _>
        };
      }

      macro_rules! Ctx {
        () => {
          AcceptedContext<Input!(), _>
        };
      }

      let left = {
        eat('`')
          + body_optional()
          + (eat('`') | eat("${").then(|ctx: Ctx!()| ctx.input.state.nested += 1))
      };

      let middle_or_right = {
        eat('}')
          + body_optional()
          + (eat('`').then(|ctx: Ctx!()| ctx.input.state.nested -= 1)
            | eat("${").then(|ctx: Ctx!()| ctx.input.state.nested += 1))
      }
      .prevent(|input| input.state.nested == 0);

      let others = (next(|c| c != '}' && c != '`')
        .prevent(|input: Input!()| input.state.nested == 0)
        | next(|c| c != '`').prevent(|input: Input!()| input.state.nested != 0))
        * (1..);

      others | left | middle_or_right
    })
    .build(s)
}

fn main() {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_js_template_str_lexer() {
    let mut lexer = build_lexer("`begin${ `${ `123` }` }end`");
    while lexer.parse().is_some() {}
    assert_eq!(lexer.instant().rest(), "");

    let mut lexer = build_lexer("`begin${ 123 }middle${ 456 }end`");
    while lexer.parse().is_some() {}
    assert_eq!(lexer.instant().rest(), "");
  }
}
