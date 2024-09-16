use super::{input::ActionInput, Action, ActionExec, ActionOutput};
use crate::kind::{MockKind, SubKind};

#[inline]
fn new_mock<'a, State, Heap, T>(
  exec: impl Fn(&mut ActionInput<&mut State, &mut Heap>) -> Option<ActionOutput<MockKind<T>>> + 'a,
) -> Action<'a, MockKind<T>, State, Heap> {
  Action {
    exec: ActionExec::new(exec),
    kind: MockKind::kind_id(),
    head: None,
    muted: false,
    literal: None,
  }
}

/// Eat at most `n` bytes from the rest of the input text.
/// `0` is ***allowed*** but be careful with infinite loops.
///
/// It's recommended to set [`Action::head`] to optimize the lex performance.
/// # Panics
/// Panics if `n` is larger than the length of [`ActionInput::rest`].
/// # Examples
/// ```
/// use whitehole::lexer::action::{Action, eat};
/// // eat at most 10 bytes
/// let a: Action<_> = eat(10);
/// ```
pub fn eat<'a, State, Heap>(n: usize) -> Action<'a, MockKind<()>, State, Heap> {
  new_mock(move |input| {
    assert!(n <= input.rest().len());
    Some(ActionOutput {
      binding: MockKind::new(()).into(),
      digested: n,
    })
  })
}

/// Eat `n` bytes from the rest of the input text.
/// `0` is ***allowed*** but be careful with infinite loops.
///
/// It's recommended to set [`Action::head`] to optimize the lex performance.
/// # Caveats
/// You should ensure that `n` is smaller than or equal to the length of [`ActionInput::rest`].
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`eat`].
/// # Examples
/// ```
/// use whitehole::lexer::action::{Action, eat_unchecked};
/// // eat 10 bytes
/// let a: Action<_> = eat_unchecked(10);
/// ```
pub fn eat_unchecked<'a, State, Heap>(n: usize) -> Action<'a, MockKind<()>, State, Heap> {
  new_mock(move |input| {
    debug_assert!(n <= input.rest().len());
    Some(ActionOutput {
      binding: MockKind::new(()).into(),
      digested: n,
    })
  })
}

/// Accept a function that eats the rest of the input text and returns the number of digested bytes.
/// The function should return `0` if the action is rejected.
///
/// It's recommended to set [`Action::head`] to optimize the lex performance.
/// # Caveats
/// The function's return value (how many bytes are digested)
/// MUST be smaller than the length of [`ActionInput::rest`].
/// # Examples
/// ```
/// use whitehole::lexer::action::{Action, simple};
/// // accept all rest characters
/// let a: Action<_> = simple(|input| input.rest().len());
/// ```
#[inline]
pub fn simple<'a, State, Heap>(
  f: impl Fn(&mut ActionInput<&mut State, &mut Heap>) -> usize + 'a,
) -> Action<'a, MockKind<()>, State, Heap> {
  new_mock(move |input| match f(input) {
    0 => None,
    digested => Some(ActionOutput {
      binding: MockKind::new(()).into(),
      digested,
    }),
  })
}

/// Provide a function that eats the rest of the input text and
/// returns the number of digested bytes and the data.
/// `0` is ***allowed*** as an accepted number of digested bytes
/// but be careful with infinite loops.
/// Return [`None`] if the action is rejected.
///
/// This is useful if you can directly yield the data in the function,
/// instead of parsing the [`content`](super::AcceptedActionOutputContext::content)
/// later using [`Action::data`].
///
/// It's recommended to set [`Action::head`] to optimize the lex performance.
/// # Caveats
/// The function's return value (how many bytes are digested)
/// MUST be smaller than the length of [`ActionInput::rest`].
/// # Examples
/// ```
/// use whitehole::kind::MockKind;
/// use whitehole::lexer::action::{Action, simple_with_data};
/// // accept all rest characters and parse them into an integer
/// let a: Action<MockKind<i32>> = simple_with_data(|input| Some((input.rest().len(), input.rest().parse().unwrap())));
/// ```
#[inline]
pub fn simple_with_data<'a, State, Heap, T>(
  f: impl Fn(&mut ActionInput<&mut State, &mut Heap>) -> Option<(usize, T)> + 'a,
) -> Action<'a, MockKind<T>, State, Heap> {
  new_mock(move |input| {
    f(input).map(|(digested, data)| ActionOutput {
      binding: MockKind::new(data).into(),
      digested,
    })
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::output::ActionOutput;

  #[test]
  fn simple_accept_all() {
    assert!(matches!(
      (simple(|input| input.text().len()).exec.raw)(
        &mut ActionInput::new("123", 0, &mut (), &mut ()).unwrap()
      )
      .unwrap()
      .digested,
      3
    ));
  }

  #[test]
  fn simple_accept_rest() {
    assert!(matches!(
      (simple(|input| input.rest().len()).exec.raw)(
        &mut ActionInput::new("123", 1, &mut (), &mut ()).unwrap()
      )
      .unwrap()
      .digested,
      2
    ));
  }

  #[test]
  fn simple_reject_on_0() {
    assert!(
      (simple(|_| 0).exec.raw)(&mut ActionInput::new("123", 0, &mut (), &mut ()).unwrap())
        .is_none()
    );
  }

  #[test]
  fn simple_option_with_data_accept() {
    let action: Action<MockKind<u32>> = simple_with_data(|input| Some((input.text().len(), 123)));
    let output = (action.exec.raw)(&mut ActionInput::new("123", 0, &mut (), &mut ()).unwrap());
    assert!(matches!(
      output,
      Some(ActionOutput {
        binding,
        digested: 3,
      }) if binding.kind().data == 123
    ));
  }

  #[test]
  fn simple_option_with_data_accept_0() {
    let action: Action<MockKind<u32>> = simple_with_data(|_| Some((0, 123)));
    let output = (action.exec.raw)(&mut ActionInput::new("123", 0, &mut (), &mut ()).unwrap());
    assert!(matches!(
      output,
      Some(ActionOutput {
        binding,
        digested: 0,
      }) if binding.kind().data == 123
    ));
  }

  #[test]
  fn simple_option_with_data_reject() {
    let action: Action<MockKind<u32>> = simple_with_data(|_| None);
    let output = (action.exec.raw)(&mut ActionInput::new("123", 0, &mut (), &mut ()).unwrap());
    assert!(output.is_none());
  }
}
