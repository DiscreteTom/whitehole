use super::{input::ActionInput, output::ActionOutput, Action};

pub struct SimpleAction<Kind, ActionState> {
  pub kind: Kind,
  pub executor: Box<dyn Fn(&mut ActionInput<ActionState>) -> usize>,
}

impl<Kind, ActionState> SimpleAction<Kind, ActionState> {
  pub fn new<F>(kind: Kind, f: F) -> Self
  where
    F: Fn(&mut ActionInput<ActionState>) -> usize + 'static,
  {
    SimpleAction {
      kind,
      executor: Box::new(f),
    }
  }
}

impl<Kind, ActionState> Action<Kind, ActionState> for SimpleAction<Kind, ActionState>
where
  Kind: Clone,
{
  fn exec(&self, input: &mut ActionInput<ActionState>) -> ActionOutput<Kind> {
    let n = (self.executor)(input);
    if n > 0 {
      ActionOutput::Accepted {
        kind: self.kind.clone(),
        buffer: input.buffer(),
        start: input.start(),
        end: input.start() + n,
      }
    } else {
      ActionOutput::Rejected
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accept_all() {
    let output = SimpleAction::new((), |input| input.buffer().len()).exec(&mut ActionInput::new(
      "123",
      0,
      &mut (),
    ));
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
  fn accept_rest() {
    let output = SimpleAction::new((), |input| input.rest().len()).exec(&mut ActionInput::new(
      "123",
      1,
      &mut (),
    ));
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
      assert_eq!(start, 1);
      assert_eq!(end, 3);
    }
  }

  #[test]
  fn reject() {
    let output = SimpleAction::new((), |_| 0).exec(&mut ActionInput::new("123", 0, &mut ()));
    assert!(matches!(output, ActionOutput::Rejected));
  }
}
