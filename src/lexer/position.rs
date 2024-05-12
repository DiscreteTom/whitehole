use super::token::Range;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
  /// A zero-based line value.
  pub line: usize,
  /// A zero-based character value.
  pub character: usize,
}

pub struct PositionTransformer {
  line_ranges: Vec<Range>,
}

impl Default for PositionTransformer {
  fn default() -> Self {
    PositionTransformer {
      line_ranges: vec![Range { start: 0, end: 0 }],
    }
  }
}

impl PositionTransformer {
  /// Create a new transformer and calculate the line ranges from the given string.
  pub fn new(string: &str) -> Self {
    let mut transformer = PositionTransformer::default();
    transformer.update(string);
    transformer
  }

  pub fn line_ranges(&self) -> &Vec<Range> {
    &self.line_ranges
  }

  /// Update [`Self::line_ranges`] with the given string.
  pub fn update(&mut self, append: &str) {
    let mut current_line_range = self.line_ranges.pop().unwrap();
    let start = current_line_range.end;
    for (i, c) in append.char_indices() {
      if c == '\n' {
        let next_line_index = start + i + 1;
        self.line_ranges.push(Range {
          start: current_line_range.start,
          end: next_line_index,
        });
        current_line_range = Range {
          start: next_line_index,
          end: next_line_index,
        };
      }
    }
    current_line_range.end = start + append.len();
    self.line_ranges.push(current_line_range);
  }

  /// Transform `0`-based index to `1`-based line and column.
  /// Return [`None`] if the index is out of range.
  pub fn transform(&self, index: usize) -> Option<Position> {
    if index >= self.line_ranges.last().unwrap().end {
      return None;
    }

    match self.line_ranges.binary_search_by(|Range { start, end }| {
      if index < *start {
        Ordering::Greater
      } else if index >= *end {
        Ordering::Less
      } else {
        Ordering::Equal
      }
    }) {
      Err(_) => None,
      Ok(line_index) => Some(Position {
        line: line_index,
        character: index - self.line_ranges[line_index].start,
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
      Some(Position {
        line: 0,
        character: 0
      })
    );
    assert_eq!(
      transformer.transform(1),
      Some(Position {
        line: 1,
        character: 0
      })
    );
    assert_eq!(
      transformer.transform(2),
      Some(Position {
        line: 2,
        character: 0
      })
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
      Some(Position {
        line: 0,
        character: 0
      })
    );
    assert_eq!(
      transformer.transform(s.find("c").unwrap()),
      Some(Position {
        line: 0,
        character: 2
      })
    );
    assert_eq!(
      transformer.transform(s.find("\n").unwrap()),
      Some(Position {
        line: 0,
        character: 3
      })
    );
    assert_eq!(
      transformer.transform(s.find("d").unwrap()),
      Some(Position {
        line: 1,
        character: 0
      })
    );
    assert_eq!(
      transformer.transform(s.find("f").unwrap()),
      Some(Position {
        line: 1,
        character: 2
      })
    );
    assert_eq!(
      transformer.transform(s.find("1").unwrap()),
      Some(Position {
        line: 2,
        character: 0
      })
    );
    assert_eq!(
      transformer.transform(s.find("5").unwrap()),
      Some(Position {
        line: 3,
        character: 2
      })
    );

    assert_eq!(transformer.transform(s.len()), None);
  }
}
