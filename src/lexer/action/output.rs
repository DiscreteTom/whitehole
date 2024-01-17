// TODO: just use Option<T>?
pub enum ActionOutput<'buffer, Kind> {
  Accepted {
    kind: Kind,
    buffer: &'buffer str,
    start: usize,
    end: usize,
  },
  Rejected,
}
