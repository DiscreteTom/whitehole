use super::input::ActionInput;

pub struct ActionOutput<Kind, ErrorType> {
  pub kind: Kind,
  pub digested: usize,
  pub muted: bool,
  pub error: Option<ErrorType>,
}

pub struct EnhancedActionOutput<'buffer, Kind, ErrorType> {
  pub kind: Kind,
  pub digested: usize,
  pub muted: bool,
  pub error: Option<ErrorType>,

  pub buffer: &'buffer str,
  pub start: usize,
}

impl<'buffer, Kind, ErrorType> EnhancedActionOutput<'buffer, Kind, ErrorType> {
  pub fn new<ActionState>(
    input: &ActionInput<'buffer, '_, ActionState>,
    output: ActionOutput<Kind, ErrorType>,
  ) -> Self {
    EnhancedActionOutput {
      kind: output.kind,
      digested: output.digested,
      muted: output.muted,
      error: output.error,

      start: input.start(),
      buffer: input.buffer(),
    }
  }

  pub fn end(&self) -> usize {
    self.start + self.digested
  }

  pub fn content(&self) -> &'buffer str {
    &self.buffer[self.start..self.end()]
  }

  pub fn rest(&self) -> &'buffer str {
    &self.buffer[self.end()..]
  }
}

impl<'buffer, Kind, ErrorType> Into<ActionOutput<Kind, ErrorType>>
  for EnhancedActionOutput<'buffer, Kind, ErrorType>
{
  fn into(self) -> ActionOutput<Kind, ErrorType> {
    ActionOutput {
      kind: self.kind,
      digested: self.digested,
      muted: self.muted,
      error: self.error,
    }
  }
}

impl<'buffer, Kind, ErrorType> Into<Option<ActionOutput<Kind, ErrorType>>>
  for EnhancedActionOutput<'buffer, Kind, ErrorType>
{
  fn into(self) -> Option<ActionOutput<Kind, ErrorType>> {
    Some(self.into())
  }
}
