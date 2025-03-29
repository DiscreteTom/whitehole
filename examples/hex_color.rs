use whitehole::{
  combinator::{eat, next},
  parser::Parser,
};

fn main() {
  let double_hex = || {
    // Repeat a combinator with `*`.
    (next(|c| c.is_ascii_hexdigit()) * 2)
      // Convert the matched content to `u8`.
      .select(|accept| u8::from_str_radix(accept.content(), 16).unwrap())
      // Wrap `u8` to `(u8,)`, this is required by `+` below.
      .tuple()
  };

  // Concat multiple combinators with `+`.
  // Tuple values will be concatenated into a single tuple.
  // Here `() + (u8,) + (u8,) + (u8,)` will be `(u8, u8, u8)`.
  let entry = eat('#') + double_hex() + double_hex() + double_hex();

  let mut parser = Parser::builder().entry(entry).build("#FFA500");
  let output = parser.next().unwrap();
  assert_eq!(output.digested, 7);
  assert_eq!(output.value, (0xFF, 0xA5, 0x00));
}

#[test]
fn with_log() {
  let double_hex = || {
    (next(|c| c.is_ascii_hexdigit()).log("hex") * 2)
      .log("double_hex")
      .select(|accept| u8::from_str_radix(accept.content(), 16).unwrap())
      .tuple()
  };

  let entry =
    (eat('#').log("hash") + double_hex().log("R") + double_hex().log("G") + double_hex().log("B"))
      .log("entry");

  let mut parser = Parser::builder().entry(entry).build("#FFA500");
  parser.next().unwrap();
}

#[test]
fn with_breakpoint() {
  let double_hex = || {
    (next(|c| c.is_ascii_hexdigit()) * 2)
      .select(|accept| u8::from_str_radix(accept.content(), 16).unwrap())
      .tuple()
  };
  // wrap the original combinator
  let double_hex = || {
    use whitehole::{action::Action, combinator::wrap};
    let c = double_hex();
    wrap(move |instant| {
      // set a breakpoint here
      // TODO: simplify this
      c.exec(Input {
        instant,
        state: &mut (),
        heap: &mut (),
      })
    })
  };

  let entry = eat('#') + double_hex() + double_hex() + double_hex();

  let mut parser = Parser::builder().entry(entry).build("#FFA500");
  parser.next().unwrap();
}
