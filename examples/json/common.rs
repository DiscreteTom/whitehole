use in_str::in_str;
use whitehole::{
  action::Action,
  combinator::{eat, next, Combinator},
};

pub fn whitespaces() -> Combinator<impl Action<Text = str, State = (), Heap = (), Value = ()>> {
  // Use `* (1..)` to repeat for one or more times.
  next(in_str!(" \t\r\n")) * (1..)
}

pub fn number() -> Combinator<impl Action<Text = str, State = (), Heap = (), Value = ()>> {
  // To re-use a combinator for multiple times, instead of wrapping the combinator in an Rc,
  // use a closure to generate the combinator for better runtime performance (via inlining).
  let digits = || next(|c| c.is_ascii_digit()) * (1..);

  let integer = {
    let digit_1_to_9 = next(|c| matches!(c, '1'..='9'));
    eat('0') | (digit_1_to_9 + digits().optional())
  };
  let fraction = eat('.') + digits();
  let exponent = (eat('e') | 'E') + (eat('-') | '+').optional() + digits();

  eat('-').optional() + integer + fraction.optional() + exponent.optional()
}

pub fn string() -> Combinator<impl Action<Text = str, State = (), Heap = (), Value = ()>> {
  let body_optional = {
    let escape = {
      let simple = next(in_str!("\"\\/bfnrt"));
      let hex = eat('u') + next(|c| c.is_ascii_hexdigit()) * 4;
      eat('\\') + (simple | hex)
    };

    let non_escape =
      next(|c| c != '"' && c != '\\' && matches!(c, '\u{0020}'..='\u{10ffff}')) * (1..);

    // Use `* (..)` to repeat for zero or more times.
    (escape | non_escape) * ..
  };
  eat('"') + body_optional + '"'
}
