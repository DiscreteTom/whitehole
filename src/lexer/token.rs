mod token_kind;

pub use token_kind::{TokenKind, TokenKindId};

#[derive(Debug)]
pub struct Range {
  /// 0-based index.
  pub start: usize,
  /// 0-based index. Exclusive.
  pub end: usize,
}

// make all fields public so the user can destruct the struct and get the fields
pub struct Token<'buffer, Kind, ErrorType> {
  /// The kind and the binding data.
  pub kind: Kind,
  pub content: &'buffer str,
  pub range: Range,
  pub error: Option<ErrorType>,
}
