use crate::common::{number, string, whitespaces};
use std::sync::LazyLock;
use whitehole::{
  action::Action,
  combinator::{eat, recur, wrap, Combinator},
};

fn wso() -> Combinator<impl Action<Value = ()>> {
  whitespaces().optional()
}

fn sep() -> Combinator<impl Action<Value = ()>> {
  eat(',') + wso()
}

pub fn parser_entry_with_recur() -> Combinator<impl Action<Value = ()>> {
  // `value` will indirectly recurse to itself, so we need to use `recur` to break the cycle.
  let (value, value_setter) = recur::<_, (), (), _>();

  // We can use `value` in `array` and `object` before it is defined.
  let array = || eat('[') + wso() + ((value() + wso()) * (..)).sep(sep()) + ']';
  let object = || {
    let object_item = string() + wso() + eat(':') + wso() + value();
    eat('{') + wso() + ((object_item + wso()) * (..)).sep(sep()) + '}'
  };

  // Finally, define `value` with `array` and `object`.
  value_setter.boxed(array() | object() | number() | string() | "true" | "false" | "null");

  whitespaces() | value()
}

pub fn parser_entry_with_static() -> Combinator<impl Action<Value = ()>> {
  fn array() -> Combinator<impl Action<Value = ()>> {
    eat('[') + wso() + ((value() + wso()) * (..)).sep(sep()) + ']'
  }

  fn object() -> Combinator<impl Action<Value = ()>> {
    let object_item = string() + wso() + eat(':') + wso() + value();
    eat('{') + wso() + ((object_item + wso()) * (..)).sep(sep()) + '}'
  }

  // `value` will indirectly recurse to itself, so we need special treatment.
  // Use `LazyLock` to create a static `Action` implementor,
  // use `Box<dyn>` to prevent recursive/infinite type.
  fn value() -> Combinator<impl Action<Value = ()>> {
    static VALUE: LazyLock<Box<dyn Action<Value = ()> + Send + Sync>> = LazyLock::new(|| {
      Box::new(array() | object() | number() | string() | "true" | "false" | "null")
    });
    wrap(|instant, ctx| VALUE.exec(instant, ctx))
  }

  whitespaces() | value()
}
