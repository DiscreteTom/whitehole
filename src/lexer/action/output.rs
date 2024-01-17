pub struct ActionOutput<Kind, ErrorType> {
  pub kind: Kind,
  pub digested: usize,
  pub muted: bool,
  pub error: Option<ErrorType>,
}

// pub struct WrappedActionOutput<'buffer, Kind, ErrorType> {
//   buffer: &'buffer str,
//   output: ActionOutput<Kind, ErrorType>,
// }

// impl<'buffer, Kind, ErrorType> WrappedActionOutput<'buffer, Kind, ErrorType> {
//   pub fn new(buffer: &'buffer str, output: ActionOutput<Kind, ErrorType>, kind: Kind) -> Self {
//     WrappedActionOutput { buffer, output }
//   }

//   pub fn kind(&self) -> &Kind {
//     &self.kind
//   }

//   pub fn buffer(&self) -> &'buffer str {
//     self.buffer
//   }

//   pub fn start(&self) -> usize {
//     self.output.start
//   }

//   pub fn end(&self) -> usize {
//     self.output.digested
//   }

//   pub fn muted(&self) -> bool {
//     self.output.muted
//   }

//   pub fn error(&self) -> Option<&ErrorType> {
//     self.output.error.as_ref()
//   }

//   pub fn content(&self) -> &'buffer str {
//     &self.buffer[self.start()..self.end()]
//   }

//   pub fn digested(&self) -> usize {
//     self.output.digested - self.output.start
//   }

//   pub fn rest(&self) -> &'buffer str {
//     &self.buffer[self.end()..]
//   }
// }
