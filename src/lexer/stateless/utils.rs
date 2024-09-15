use super::head_map::RuntimeActions;
use crate::lexer::{
  action::{ActionInput, ActionOutput},
  re_lex::ReLexContext,
};

/// Traverse all actions with a mutable input to find the first accepted action.
/// Return the output, the index of the accepted action and whether the action is muted.
/// If no accepted action, return [`None`].
pub(super) fn traverse_actions<Kind, State, Heap>(
  input: &mut ActionInput<&mut State, &mut Heap>,
  actions: &RuntimeActions<Kind, State, Heap>,
  re_lex: &ReLexContext,
) -> Option<(ActionOutput<Kind>, usize, bool)> {
  for (i, exec) in actions
    .execs()
    .iter()
    .enumerate()
    .skip(if input.start() == re_lex.start {
      // SAFETY: it is ok that if `skip` is larger than `actions.len()`
      re_lex.skip
    } else {
      0
    })
  {
    if let Some(output) = (exec.raw)(input) {
      debug_assert!(output.digested <= input.rest().len());
      // return once accepted action is found
      return Some((output, i, unsafe {
        // SAFETY: `actions.exec` and `actions.muted` have the same length
        // so `i` is a valid index for `actions.muted`
        *actions.muted().get_unchecked(i)
      }));
    }
  }

  // no accepted action
  None
}

/// Break the loop if the value is [`None`],
/// otherwise return the value.
macro_rules! break_loop_on_none {
  ($e:expr) => {
    match $e {
      Some(v) => v,
      None => break,
    }
  };
}
pub(super) use break_loop_on_none;

/// Prepare the input for lexing.
macro_rules! prepare_input {
  ($text:expr, $digested:expr, $options:ident) => {
    ActionInput::new(
      $text,
      $options.start + $digested,
      &mut *$options.state,
      &mut *$options.heap,
    )
  };
}
pub(super) use prepare_input;

/// Lex with the given input and actions.
/// Break the loop if the output is [`None`],
/// otherwise update the `digested` length and return the lex result.
macro_rules! lex {
  ($input:expr, $actions:expr, $re_lex:expr, $digested:expr) => {{
    let res = break_loop_on_none!(traverse_actions(&mut $input, $actions, $re_lex));
    $digested += res.0.digested;
    res
  }};
}
pub(super) use lex;

// TODO: add tests
