use super::{DefaultSubKind, SubKind};

/// This implements [`SubKind`] and [`DefaultSubKind`],
/// and the [`SubKindId::value`](crate::kind::SubKindId::value)
/// will always be `0`.
/// This is useful as a placeholder or data carrier.
/// # Examples
/// ```
/// use whitehole::kind::{MockKind, SubKind, KindIdBinding};
///
/// let v1: KindIdBinding<MockKind<i32>> = MockKind::new(42).into();
/// let v2: KindIdBinding<MockKind<bool>> = MockKind::new(true).into();
///
/// assert_eq!(v1.id(), MockKind::kind_id());
/// assert_eq!(v2.id(), MockKind::kind_id());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct MockKind<T> {
  /// The data carried by the [`MockKind`].
  pub data: T,
}

impl<T> MockKind<T> {
  /// Create a new instance with the given data.
  #[inline]
  pub const fn new(data: T) -> Self {
    Self { data }
  }
}

impl<T> SubKind for MockKind<T> {
  type Kind = Self;
  const VARIANT_INDEX: usize = 0;
}

impl<T> DefaultSubKind for MockKind<T> {
  type Default = Self;
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::kind::KindIdBinding;

  #[test]
  fn mock_kind_new() {
    assert_eq!(MockKind::new(42).data, 42);
    assert_eq!(MockKind::new("123").data, "123");
  }

  #[test]
  fn mock_kind_id() {
    assert_eq!(MockKind::<u32>::kind_id().value(), 0);
    assert_eq!(MockKind::<Box<u32>>::kind_id().value(), 0);
  }

  #[test]
  fn mock_kind_into_binding() {
    let v1: KindIdBinding<MockKind<i32>> = MockKind::new(42).into();
    let v2: KindIdBinding<MockKind<bool>> = MockKind::new(true).into();

    assert_eq!(v1.id(), MockKind::kind_id());
    assert_eq!(v2.id(), MockKind::kind_id());
  }

  #[test]
  fn default_mock_kind_id_binding() {
    let v1: KindIdBinding<MockKind<i32>> = Default::default();
    let v2: KindIdBinding<MockKind<bool>> = Default::default();

    assert_eq!(v1.id(), MockKind::kind_id());
    assert_eq!(v1.kind().data, 0);
    assert_eq!(v2.id(), MockKind::kind_id());
    assert!(!v2.kind().data);
  }
}
