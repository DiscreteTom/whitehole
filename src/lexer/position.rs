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
  /// See [`Self::line_ranges`].
  line_ranges: Vec<Range>,
}

impl Default for PositionTransformer {
  #[inline]
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

  /// Return the byte ranges of each line.
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

  /// Return the line index at the given `index` by byte.
  /// Return [`None`] if the `index` is out of known line ranges.
  pub fn line_index_at(&self, index: usize) -> Option<usize> {
    self
      .line_ranges
      .binary_search_by(|Range { start, end }| {
        if index < *start {
          Ordering::Greater
        } else if index >= *end {
          Ordering::Less
        } else {
          Ordering::Equal
        }
      })
      .ok()
  }

  /// Transform zero-based index to [`Position`].
  /// Return [`None`] if the `index` is out of known line ranges.
  /// This is ideal for single transformation, but not for batch transformation.
  /// The caller should make sure the index is smaller than the text length.
  pub fn transform(&self, index: usize, text: &str) -> Option<Position> {
    if index >= self.line_ranges.last().unwrap().end {
      return None;
    }

    debug_assert!(index < text.len());

    self.line_index_at(index).map(|line_index| {
      let line = &text[Range {
        start: self.line_ranges[line_index].start,
        end: index,
      }];

      Position {
        line: line_index,
        // `line` contains all chars before the target char
        // so we can use `line.chars().count()` to get the character index, without `-1`
        character: line.chars().count(),
      }
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default() {
    let text = "";
    let transformer = PositionTransformer::default();
    assert_eq!(transformer.transform(0, text), None);
  }

  #[test]
  fn empty() {
    let text = "";
    let mut transformer = PositionTransformer::default();
    transformer.update(text);
    assert_eq!(transformer.transform(0, text), None);
  }

  #[test]
  fn new_line_only() {
    let text = "\n\n\n";
    let mut transformer = PositionTransformer::default();
    transformer.update(text);
    assert_eq!(
      transformer.transform(0, text),
      Some(Position {
        line: 0,
        character: 0
      })
    );
    assert_eq!(
      transformer.transform(1, text),
      Some(Position {
        line: 1,
        character: 0
      })
    );
    assert_eq!(
      transformer.transform(2, text),
      Some(Position {
        line: 2,
        character: 0
      })
    );
    assert_eq!(transformer.transform(3, text), None);
  }

  #[test]
  fn complex() {
    let mut transformer = PositionTransformer::default();
    let text = "abc\ndef\n123\n345";
    transformer.update(text);
    assert_eq!(
      transformer.transform(text.find('a').unwrap(), text),
      Some(Position {
        line: 0,
        character: 0
      })
    );
    assert_eq!(
      transformer.transform(text.find('c').unwrap(), text),
      Some(Position {
        line: 0,
        character: 2
      })
    );
    assert_eq!(
      transformer.transform(text.find('\n').unwrap(), text),
      Some(Position {
        line: 0,
        character: 3
      })
    );
    assert_eq!(
      transformer.transform(text.find('d').unwrap(), text),
      Some(Position {
        line: 1,
        character: 0
      })
    );
    assert_eq!(
      transformer.transform(text.find('f').unwrap(), text),
      Some(Position {
        line: 1,
        character: 2
      })
    );
    assert_eq!(
      transformer.transform(text.find('1').unwrap(), text),
      Some(Position {
        line: 2,
        character: 0
      })
    );
    assert_eq!(
      transformer.transform(text.find('5').unwrap(), text),
      Some(Position {
        line: 3,
        character: 2
      })
    );

    assert_eq!(transformer.transform(text.len(), text), None);
  }

  #[test]
  fn utf8() {
    let mut transformer = PositionTransformer::default();
    let text = "a\nb🦀c哈";
    transformer.update(text);
    assert_eq!(
      transformer.transform(text.find('a').unwrap(), text),
      Some(Position {
        line: 0,
        character: 0
      })
    );
    assert_eq!(
      transformer.transform(text.find('\n').unwrap(), text),
      Some(Position {
        line: 0,
        character: 1
      })
    );
    assert_eq!(
      transformer.transform(text.find('b').unwrap(), text),
      Some(Position {
        line: 1,
        character: 0
      })
    );
    assert_eq!(
      transformer.transform(text.find('🦀').unwrap(), text),
      Some(Position {
        line: 1,
        character: 1
      })
    );
    assert_eq!(
      transformer.transform(text.find('c').unwrap(), text),
      Some(Position {
        line: 1,
        character: 2
      })
    );
    assert_eq!(
      transformer.transform(text.find('哈').unwrap(), text),
      Some(Position {
        line: 1,
        character: 3
      })
    );
  }
}
