use super::{input::ActionInput, output::ActionOutput, Action};
use regex::Regex;

pub struct RegexAction<Kind> {
  pub kind: Kind,
  pub re: Regex,
}

impl<Kind> RegexAction<Kind> {
  pub fn new(kind: Kind, re: &str) -> Self {
    RegexAction {
      kind,
      re: Regex::new(re).unwrap(),
    }
  }
}

impl<'action, Kind, ActionState> Action<'action, Kind, ActionState> for RegexAction<Kind>
where
  Kind: Clone,
{
  fn exec(&self, input: &'action ActionInput<ActionState>) -> ActionOutput<Kind> {
    match self.re.find(input.rest()) {
      Some(m) => ActionOutput::Accepted {
        kind: self.kind.clone(),
        buffer: input.buffer,
        start: input.start + m.start(),
        end: input.start + m.end(),
      },
      None => ActionOutput::Rejected,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Clone)]
  enum MyKind {
    Simple,
  }

  #[test]
  fn regex_start() {
    let action = RegexAction::new(MyKind::Simple, r"^\d+");
    let input = ActionInput {
      buffer: "123",
      start: 0,
      state: &(),
    };
    let output = action.exec(&input);
    assert!(matches!(output, ActionOutput::Accepted { .. }));
    if let ActionOutput::Accepted {
      kind,
      buffer,
      start,
      end,
    } = output
    {
      assert!(matches!(kind, MyKind::Simple));
      assert_eq!(buffer, "123");
      assert_eq!(start, 0);
      assert_eq!(end, 3);
    }
  }

  #[test]
  fn regex_middle() {
    let action = RegexAction::new(MyKind::Simple, r"^\d+");
    let input = ActionInput {
      buffer: "abc123",
      start: 3,
      state: &(),
    };
    let output = action.exec(&input);
    assert!(matches!(output, ActionOutput::Accepted { .. }));
    if let ActionOutput::Accepted {
      kind,
      buffer,
      start,
      end,
    } = output
    {
      assert!(matches!(kind, MyKind::Simple));
      assert_eq!(buffer, "abc123");
      assert_eq!(start, 3);
      assert_eq!(end, 6);
    }
  }
}
