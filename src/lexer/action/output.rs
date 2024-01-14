pub enum ActionOutput<Kind> {
  Accepted {
    kind: Kind,
    buffer: &'static str,
    start: usize,
    end: usize,
  },
  Rejected,
}
