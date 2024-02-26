pub enum Associativity {
  LeftToRight,
  RightToLeft,
}

pub struct GroupAssociativity<T> {
  pub associativity: Associativity,
  pub group: Vec<T>,
}

pub fn left_to_right<T>(group: impl Into<Vec<T>>) -> GroupAssociativity<T> {
  GroupAssociativity {
    associativity: Associativity::LeftToRight,
    group: group.into(),
  }
}

pub fn right_to_left<T>(group: impl Into<Vec<T>>) -> GroupAssociativity<T> {
  GroupAssociativity {
    associativity: Associativity::RightToLeft,
    group: group.into(),
  }
}

impl<T, const N: usize> From<[T; N]> for GroupAssociativity<T> {
  fn from(group: [T; N]) -> Self {
    Self {
      associativity: Associativity::LeftToRight,
      group: group.into(),
    }
  }
}

pub struct Priority<T>(pub Vec<GroupAssociativity<T>>);

impl<T, const N: usize> From<[GroupAssociativity<T>; N]> for Priority<T> {
  fn from(groups: [GroupAssociativity<T>; N]) -> Self {
    Self(groups.into())
  }
}

impl<T, const M: usize, const N: usize> From<[[T; M]; N]> for Priority<T> {
  fn from(groups: [[T; M]; N]) -> Self {
    Self(groups.into_iter().map(|x| x.into()).collect())
  }
}
