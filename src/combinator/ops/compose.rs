use crate::combinator::{bytes, Eat};

pub trait ComposeLiteral<Rhs> {
  type Output;

  fn to(rhs: Rhs) -> Self::Output;
}

impl ComposeLiteral<char> for str {
  type Output = Eat<char>;

  #[inline]
  fn to(rhs: char) -> Self::Output {
    Eat::new(rhs)
  }
}

impl ComposeLiteral<String> for str {
  type Output = Eat<String>;

  #[inline]
  fn to(rhs: String) -> Self::Output {
    Eat::new(rhs)
  }
}

impl<'a> ComposeLiteral<&'a str> for str {
  type Output = Eat<&'a str>;

  #[inline]
  fn to(rhs: &'a str) -> Self::Output {
    Eat::new(rhs)
  }
}

impl ComposeLiteral<u8> for [u8] {
  type Output = bytes::Eat<u8>;

  #[inline]
  fn to(rhs: u8) -> Self::Output {
    bytes::Eat::new(rhs)
  }
}

impl ComposeLiteral<Vec<u8>> for [u8] {
  type Output = bytes::Eat<Vec<u8>>;

  #[inline]
  fn to(rhs: Vec<u8>) -> Self::Output {
    bytes::Eat::new(rhs)
  }
}

impl<'a> ComposeLiteral<&'a [u8]> for [u8] {
  type Output = bytes::Eat<&'a [u8]>;

  #[inline]
  fn to(rhs: &'a [u8]) -> Self::Output {
    bytes::Eat::new(rhs)
  }
}

impl<'a, const N: usize> ComposeLiteral<&'a [u8; N]> for [u8] {
  type Output = bytes::Eat<&'a [u8; N]>;

  #[inline]
  fn to(rhs: &'a [u8; N]) -> Self::Output {
    bytes::Eat::new(rhs)
  }
}
