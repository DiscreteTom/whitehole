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

impl<Kind, ActionState> Action<Kind, ActionState> for RegexAction<Kind>
where
  Kind: Clone,
{
  fn exec(&self, input: &mut ActionInput<ActionState>) -> ActionOutput<Kind> {
    match self.re.find(input.rest()) {
      Some(m) => ActionOutput::Accepted {
        kind: self.kind.clone(),
        buffer: input.buffer(),
        start: input.start() + m.start(),
        end: input.start() + m.end(),
      },
      None => ActionOutput::Rejected,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn regex_start() {
    let output = RegexAction::new((), r"^\d+").exec(&mut ActionInput::new("123", 0, &mut ()));
    assert!(matches!(output, ActionOutput::Accepted { .. }));
    if let ActionOutput::Accepted {
      kind,
      buffer,
      start,
      end,
    } = output
    {
      assert_eq!(kind, ());
      assert_eq!(buffer, "123");
      assert_eq!(start, 0);
      assert_eq!(end, 3);
    }
  }

  #[test]
  fn regex_middle() {
    let output = RegexAction::new((), r"^\d+").exec(&mut (ActionInput::new("abc123", 3, &mut ())));
    assert!(matches!(output, ActionOutput::Accepted { .. }));
    if let ActionOutput::Accepted {
      kind,
      buffer,
      start,
      end,
    } = output
    {
      assert_eq!(kind, ());
      assert_eq!(buffer, "abc123");
      assert_eq!(start, 3);
      assert_eq!(end, 6);
    }
  }
}
