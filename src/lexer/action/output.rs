use super::input::ActionInput;

pub struct ActionOutput<Kind, ErrorType> {
  pub kind: Kind,
  /// How many characters are digested by this action.
  pub digested: usize,
  /// If `true`, the action is accepted but no token is emitted,
  /// and the lexing process will continue.
  pub muted: bool,
  pub error: Option<ErrorType>,
}

pub struct EnhancedActionOutput<'buffer, Kind, ErrorType> {
  /// The original [ActionOutput].
  pub raw: ActionOutput<Kind, ErrorType>,
  /// [ActionInput.buffer](ActionInput).
  pub buffer: &'buffer str,
  /// [ActionInput.start](ActionInput).
  pub start: usize,
}

impl<'buffer, Kind, ErrorType> EnhancedActionOutput<'buffer, Kind, ErrorType> {
  pub fn new<ActionState>(
    input: &ActionInput<'buffer, '_, ActionState>,
    output: ActionOutput<Kind, ErrorType>,
  ) -> Self {
    EnhancedActionOutput {
      raw: output,
      start: input.start(),
      buffer: input.buffer(),
    }
  }

  // re-export output fields
  pub fn kind(&self) -> &Kind {
    &self.raw.kind
  }
  pub fn digested(&self) -> usize {
    self.raw.digested
  }
  pub fn muted(&self) -> bool {
    self.raw.muted
  }
  pub fn error(&self) -> &Option<ErrorType> {
    &self.raw.error
  }

  pub fn end(&self) -> usize {
    self.start + self.digested()
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
    self.raw
  }
}

impl<'buffer, Kind, ErrorType> Into<Option<ActionOutput<Kind, ErrorType>>>
  for EnhancedActionOutput<'buffer, Kind, ErrorType>
{
  fn into(self) -> Option<ActionOutput<Kind, ErrorType>> {
    Some(self.into())
  }
}
