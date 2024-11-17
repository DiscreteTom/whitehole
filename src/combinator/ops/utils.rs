use crate::parse::{Input, Output, Parse};
use std::marker::PhantomData;

// TODO: move to eat, remove dup code

/// Created when performing `+` or `|` by [`Combinator`](crate::combinator::Combinator) with [`char`].
/// Similar to [`eat(char)`](crate::combinator::eat).
///
/// This struct exists because the output of operator overloading has to be a concrete type.
/// You shouldn't use this struct directly.
#[derive(Debug, Clone, Copy)]
pub struct EatChar<State, Heap> {
  c: char,
  _phantom: PhantomData<(State, Heap)>,
}

impl<State, Heap> EatChar<State, Heap> {
  #[inline]
  pub fn new(c: char) -> Self {
    Self {
      c,
      _phantom: PhantomData,
    }
  }
}

impl<State, Heap> Parse<State, Heap> for EatChar<State, Heap> {
  type Kind = ();

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    input
      .rest()
      .starts_with(self.c)
      .then(|| unsafe { input.digest_unchecked(self.c.len_utf8()) })
  }
}

/// Created when performing `+` or `|` by [`Combinator`](crate::combinator::Combinator) with [`String`].
/// Similar to [`eat(String)`](crate::combinator::eat).
///
/// This struct exists because the output of operator overloading has to be a concrete type.
/// You shouldn't use this struct directly.
#[derive(Debug, Clone)]
pub struct EatString<State, Heap> {
  s: String,
  _phantom: PhantomData<(State, Heap)>,
}

impl<State, Heap> EatString<State, Heap> {
  #[inline]
  pub fn new(s: String) -> Self {
    Self {
      s,
      _phantom: PhantomData,
    }
  }
}

impl<State, Heap> Parse<State, Heap> for EatString<State, Heap> {
  type Kind = ();

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    input
      .rest()
      .starts_with(&self.s)
      .then(|| unsafe { input.digest_unchecked(self.s.len()) })
  }
}

/// Created when performing `+` or `|` by [`Combinator`](crate::combinator::Combinator) with `&str`.
/// Similar to [`eat(&str)`](crate::combinator::eat).
///
/// This struct exists because the output of operator overloading has to be a concrete type.
/// You shouldn't use this struct directly.
#[derive(Debug, Clone, Copy)]
pub struct EatStr<'a, State, Heap> {
  s: &'a str,
  _phantom: PhantomData<(State, Heap)>,
}

impl<'a, State, Heap> EatStr<'a, State, Heap> {
  #[inline]
  pub fn new(s: &'a str) -> Self {
    Self {
      s,
      _phantom: PhantomData,
    }
  }
}

impl<'a, State, Heap> Parse<State, Heap> for EatStr<'a, State, Heap> {
  type Kind = ();

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    input
      .rest()
      .starts_with(self.s)
      .then(|| unsafe { input.digest_unchecked(self.s.len()) })
  }
}

/// Created when performing `+` or `|` by [`Combinator`](crate::combinator::Combinator) with [`usize`].
/// Similar to [`eat(usize)`](crate::combinator::eat).
///
/// This struct exists because the output of operator overloading has to be a concrete type.
/// You shouldn't use this struct directly.
#[derive(Debug, Clone, Copy)]
pub struct EatUsize<State, Heap> {
  u: usize,
  _phantom: PhantomData<(State, Heap)>,
}

impl<State, Heap> EatUsize<State, Heap> {
  #[inline]
  pub fn new(u: usize) -> Self {
    Self {
      u,
      _phantom: PhantomData,
    }
  }
}

impl<State, Heap> Parse<State, Heap> for EatUsize<State, Heap> {
  type Kind = ();

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    input.digest(self.u)
  }
}

// TODO: tests
