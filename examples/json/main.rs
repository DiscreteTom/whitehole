mod common;
mod lexer;
mod parser;

use crate::{
  lexer::lexer_entry,
  parser::{parser_entry_with_recur, parser_entry_with_static},
};
use whitehole::{action::Action, combinator::Combinator, parser::Parser};

const TEXT: &str = r#"
{
  "name": "John Doe",
  "age": 30,
  "is_student": false,
  "scores": [100, 90, 80],
  "address": {
    "city": "New York",
    "zip": "10001"
  }
}
"#;

fn print_all_with_range(entry: Combinator<impl Action<Value = ()>>) {
  let mut parser = Parser::builder().entry(entry.range()).build(TEXT);

  loop {
    let output = parser.parse();
    if let Some(node) = output {
      println!(
        "{}..{}: {:?}",
        node.value.range.start,
        node.value.range.end,
        &TEXT[node.value.range.clone()]
      );
    } else {
      break;
    }
  }

  let rest = parser.instant().rest();
  if !rest.is_empty() {
    panic!(
      "failed to consume the whole input, remaining: {:?}",
      &rest[..100.min(rest.len())]
    );
  }
}

fn main() {
  println!("lexer:");
  print_all_with_range(lexer_entry());

  println!("\nparser with recur:");
  print_all_with_range(parser_entry_with_recur());

  println!("\nparser with static:");
  print_all_with_range(parser_entry_with_static());
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_json_lexer() {
    print_all_with_range(lexer_entry());
  }

  #[test]
  fn test_json_parser() {
    print_all_with_range(parser_entry_with_recur());
    print_all_with_range(parser_entry_with_static());
  }
}
