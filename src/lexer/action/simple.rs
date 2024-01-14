use super::{input::ActionInput, output::ActionOutput, Action};

pub struct SimpleAction<Kind, ActionState>
where
  ActionState: 'static,
{
  pub kind: Kind,
  pub executor: Box<dyn Fn(&'static ActionInput<ActionState>) -> usize>,
}

impl<Kind, ActionState> SimpleAction<Kind, ActionState> {
  pub fn new<F>(kind: Kind, f: F) -> Self
  where
    F: Fn(&'static ActionInput<ActionState>) -> usize + 'static,
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
  ActionState: 'static,
{
  fn exec(&self, input: &'static ActionInput<ActionState>) -> ActionOutput<Kind> {
    let n = (self.executor)(input);
    if n > 0 {
      ActionOutput::Accepted {
        kind: self.kind.clone(),
        buffer: input.buffer,
        start: input.start,
        end: input.start + n,
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
    let output = SimpleAction::new((), |input| input.buffer.len()).exec(
      &(ActionInput {
        buffer: "123",
        start: 0,
        state: &(),
      }),
    );
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
}
