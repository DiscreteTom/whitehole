pub enum Associativity {
  LeftToRight,
  RightToLeft,
}

pub struct GroupAssociativity<T> {
  pub associativity: Associativity,
  pub group: Vec<T>,
}

impl<T> From<T> for GroupAssociativity<T> {
  fn from(group: T) -> Self {
    Self {
      associativity: Associativity::LeftToRight,
      group: vec![group],
    }
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

// TODO: optimize the rule
#[macro_export]
macro_rules! priority {
  ($pb:ident: $($x:expr),*) => {
      let mut $pb = $pb.priority([$(whitehole::parser::elr::builder::priority::GroupAssociativity::from($x)),*]);
  };
}

#[macro_export]
macro_rules! left_to_right {
  ($($x:expr),*) => {
    whitehole::parser::elr::builder::priority::GroupAssociativity {
      associativity: whitehole::parser::elr::builder::priority::Associativity::LeftToRight,
      group: [$($x),*].into(),
    }
  };
}

#[macro_export]
macro_rules! right_to_left {
  ($($x:expr),*) => {
    whitehole::parser::elr::builder::priority::GroupAssociativity {
      associativity: whitehole::parser::elr::builder::priority::Associativity::RightToLeft,
      group: [$($x),*].into(),
    }
  };
}
