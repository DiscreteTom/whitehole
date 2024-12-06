use std::{cell::OnceCell, rc::Rc};
use whitehole::{
  combinator::{eat, next, Combinator},
  parse::{Input, Output, Parse},
  parser::Builder,
};

// TODO: comments

type RcInner<Kind, State, Heap> =
  Rc<OnceCell<Box<dyn Parse<Kind = Kind, State = State, Heap = Heap>>>>;

pub struct RcCombinatorSetter<Kind, State, Heap> {
  rc: RcInner<Kind, State, Heap>,
}

impl<Kind, State, Heap> Default for RcCombinatorSetter<Kind, State, Heap> {
  fn default() -> Self {
    Self::new()
  }
}

impl<Kind, State, Heap> RcCombinatorSetter<Kind, State, Heap> {
  pub fn new() -> Self {
    Self {
      rc: Rc::new(OnceCell::new()),
    }
  }

  pub fn set(self, parser: Box<dyn Parse<Kind = Kind, State = State, Heap = Heap>>) {
    self.rc.set(parser).ok();
  }

  pub fn boxed(self, p: impl Parse<Kind = Kind, State = State, Heap = Heap> + 'static) {
    self.set(Box::new(p));
  }
}

pub struct RcParse<Kind, State, Heap> {
  rc: RcInner<Kind, State, Heap>,
}

impl<Kind, State, Heap> Parse for RcParse<Kind, State, Heap> {
  type Kind = Kind;
  type State = State;
  type Heap = Heap;

  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    unsafe { self.rc.get().unwrap_unchecked() }.parse(input)
  }
}

pub fn rc<Kind, State, Heap>() -> (
  impl Fn() -> Combinator<RcParse<Kind, State, Heap>>,
  RcCombinatorSetter<Kind, State, Heap>,
) {
  let setter = RcCombinatorSetter::new();
  let getter = {
    let rc = setter.rc.clone();
    move || Combinator::new(RcParse { rc: rc.clone() })
  };
  (getter, setter)
}

fn main() {
  let mut parser = Builder::new()
    .entry({
      let number = next(|c| c.is_ascii_digit()) * (1..);
      let (exp, exp_setter) = rc();
      exp_setter.boxed(number | (eat('-') + exp()));

      exp()
    })
    .build("----123");

  while let Some(node) = parser.parse() {
    println!("{:?}", node);
  }

  let rest = parser.instant().rest();
  if rest.is_empty() {
    println!("Parsing successful!");
  } else {
    println!("Parsing failed!");
    println!(
      "Parsing failed, remaining: {:?}",
      &rest[..100.min(rest.len())]
    );
  }
}
