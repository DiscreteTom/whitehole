use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
  /// 1-based line number.
  pub line: usize,
  /// 1-based column number.
  pub column: usize,
}

#[derive(Debug)]
pub struct Range {
  /// 0-based index.
  from: usize,
  /// 0-based index. Exclusive.
  to: usize,
}

pub struct PositionTransformer {
  line_ranges: Vec<Range>,
}

impl Default for PositionTransformer {
  fn default() -> Self {
    PositionTransformer {
      line_ranges: vec![Range { from: 0, to: 0 }],
    }
  }
}

impl PositionTransformer {
  pub fn new(string: &str) -> Self {
    let mut transformer = PositionTransformer::default();
    transformer.update(string);
    transformer
  }

  pub fn line_ranges(&self) -> &[Range] {
    &self.line_ranges
  }

  pub fn update(&mut self, digested: &str) {
    let mut current_line_range = self.line_ranges.pop().unwrap();
    let start = current_line_range.to;
    for (i, c) in digested.char_indices() {
      if c == '\n' {
        let next_line_index = start + i + 1;
        self.line_ranges.push(Range {
          from: current_line_range.from,
          to: next_line_index,
        });
        current_line_range = Range {
          from: next_line_index,
          to: next_line_index,
        };
      }
    }
    current_line_range.to = start + digested.len();
    self.line_ranges.push(current_line_range);
  }

  /// Transform 0-based index to 1-based line and column.
  pub fn transform(&self, index: usize) -> Option<Position> {
    match self.line_ranges.binary_search_by(|Range { from, to }| {
      if index < *from {
        Ordering::Greater
      } else if index >= *to {
        Ordering::Less
      } else {
        Ordering::Equal
      }
    }) {
      Err(_) => None,
      Ok(line_index) => Some(Position {
        line: line_index + 1,
        column: index - self.line_ranges[line_index].from + 1,
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default() {
    let transformer = PositionTransformer::default();
    assert_eq!(transformer.transform(0), None);
  }

  #[test]
  fn empty() {
    let mut transformer = PositionTransformer::default();
    transformer.update("");
    assert_eq!(transformer.transform(0), None);
  }

  #[test]
  fn new_line_only() {
    let mut transformer = PositionTransformer::default();
    transformer.update("\n\n\n");
    assert_eq!(
      transformer.transform(0),
      Some(Position { line: 1, column: 1 })
    );
    assert_eq!(
      transformer.transform(1),
      Some(Position { line: 2, column: 1 })
    );
    assert_eq!(
      transformer.transform(2),
      Some(Position { line: 3, column: 1 })
    );
    assert_eq!(transformer.transform(3), None);
  }

  #[test]
  fn complex() {
    let mut transformer = PositionTransformer::default();
    let s = "abc\ndef\n123\n345";
    transformer.update(s);
    assert_eq!(
      transformer.transform(s.find("a").unwrap()),
      Some(Position { line: 1, column: 1 })
    );
    assert_eq!(
      transformer.transform(s.find("c").unwrap()),
      Some(Position { line: 1, column: 3 })
    );
    assert_eq!(
      transformer.transform(s.find("\n").unwrap()),
      Some(Position { line: 1, column: 4 })
    );
    assert_eq!(
      transformer.transform(s.find("d").unwrap()),
      Some(Position { line: 2, column: 1 })
    );
    assert_eq!(
      transformer.transform(s.find("f").unwrap()),
      Some(Position { line: 2, column: 3 })
    );
    assert_eq!(
      transformer.transform(s.find("1").unwrap()),
      Some(Position { line: 3, column: 1 })
    );
    assert_eq!(
      transformer.transform(s.find("5").unwrap()),
      Some(Position { line: 4, column: 3 })
    );

    assert_eq!(transformer.transform(s.len()), None);
  }
}
