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

macro_rules! break_loop_on_none {
  ($e:expr) => {
    match $e {
      Some(v) => v,
      None => break,
    }
  };
}
pub(super) use break_loop_on_none;

// TODO: add tests
