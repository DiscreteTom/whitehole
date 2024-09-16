use crate::{
  lexer::action::{HeadMatcher, RcAction, RcActionExec},
  utils::lookup::{
    char::{SparseCharLookupTable, SparseCharLookupTableBuilder},
    Lookup,
  },
};

/// A layout optimized collection of [`Action`](crate::lexer::action::Action)s for runtime evaluation.
/// As per data oriented design, we store
/// [`Action::exec`](crate::lexer::action::Action::exec) and [`Action::muted`](crate::lexer::action::Action::muted)
/// in separate lists, and discard all other fields to optimize cache performance.
#[derive(Debug)]
pub(super) struct RuntimeActions<'a, Kind, State, Heap> {
  execs: Vec<RcActionExec<'a, Kind, State, Heap>>,
  // logically using a bit vec should be better than `Vec<bool>`.
  // however in head map the length of this vector is usually 1,
  // so the overhead of using a bit vec is not worth it.
  muted: Vec<bool>,
}

impl<'a, Kind, State, Heap> Default for RuntimeActions<'a, Kind, State, Heap> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<'a, Kind, State, Heap> Clone for RuntimeActions<'a, Kind, State, Heap> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      execs: self.execs.clone(),
      muted: self.muted.clone(),
    }
  }
}

impl<'a, Kind, State, Heap> RuntimeActions<'a, Kind, State, Heap> {
  #[inline]
  pub fn new() -> Self {
    Self {
      // in head map maybe every head only has one action
      // but there will be at least one action, so we allocate 1 here
      execs: Vec::with_capacity(1),
      muted: Vec::with_capacity(1),
    }
  }

  // getters
  #[inline]
  pub fn execs(&self) -> &Vec<RcActionExec<'a, Kind, State, Heap>> {
    &self.execs
  }
  #[inline]
  pub const fn muted(&self) -> &Vec<bool> {
    &self.muted
  }

  #[inline]
  pub fn push(&mut self, exec: RcActionExec<'a, Kind, State, Heap>, muted: bool) {
    self.execs.push(exec);
    self.muted.push(muted);
  }

  #[inline]
  pub fn len(&self) -> usize {
    // `self.execs` and `self.muted` should have the same length
    self.execs.len()
  }
}

#[derive(Debug)]
pub(super) struct HeadMap<'a, Kind, State, Heap> {
  /// Store actions for known chars.
  known_map: SparseCharLookupTable<RuntimeActions<'a, Kind, State, Heap>>,
  /// Store actions for unknown chars.
  unknown_fallback: RuntimeActions<'a, Kind, State, Heap>,
}

/// A new-type to represent the return type of [`HeadMap::collect_all_known`].
/// This is to prevent other modules from modifying the known map by mistake
/// before calling [`HeadMap::new`].
pub(super) struct KnownHeadChars<'a, Kind, State, Heap>(
  SparseCharLookupTableBuilder<RuntimeActions<'a, Kind, State, Heap>>,
);

impl<'a, Kind, State, Heap> Clone for KnownHeadChars<'a, Kind, State, Heap> {
  #[inline]
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<'a, Kind, State, Heap> HeadMap<'a, Kind, State, Heap> {
  /// Collect all known head chars from all actions instead of a subset of actions to make sure
  /// 'known' has a consistent meaning across all head maps in a stateless lexer
  /// (otherwise maybe only a subset of chars are known for a subset of actions,
  /// in which case the 'known' has an inconsistent meaning).
  /// This must be done before creating a head map
  /// because we need to iter over all known chars when filling the head map
  /// with [`HeadMatcher::Not`] and [`HeadMatcher::Unknown`].
  #[inline] // there is only one call site, so mark this as inline
  pub fn collect_all_known(
    actions: &[RcAction<'a, Kind, State, Heap>],
  ) -> KnownHeadChars<'a, Kind, State, Heap> {
    // assume every action has at least one known head char.
    // we don't need to deduplicate chars here
    // since SparseCharLookupTableBuilder will deduplicate them (and maybe faster).
    let mut known_chars = Vec::with_capacity(actions.len());

    for (_, p) in actions {
      if let Some(head) = p.head() {
        for c in match head {
          HeadMatcher::OneOf(set) | HeadMatcher::Not(set) => set,
          HeadMatcher::Unknown => continue,
        } {
          known_chars.push(*c);
        }
      }
    }

    KnownHeadChars(SparseCharLookupTableBuilder::new(known_chars))
  }

  /// Create a new instance with a subset of actions and a known char map created by [`Self::collect_all_known`].
  pub fn new(
    actions: &[RcAction<'a, Kind, State, Heap>],
    known_map: KnownHeadChars<'a, Kind, State, Heap>,
  ) -> Self {
    let mut unknown_fallback = RuntimeActions::new();
    let mut known_map = known_map.0;

    // fill the head map
    for (e, p) in actions {
      // when lexing the lexer needs to check the head matcher no matter the action is muted or not
      // so we won't check if the action is muted here
      if let Some(head) = p.head() {
        match head {
          HeadMatcher::OneOf(set) => {
            for c in set {
              // SAFETY: the key must exist because we have collected all known chars in `collect_all_known`
              // and `KnownHeadChars` ensures the known map is not modified before creating the head map
              unsafe { known_map.get_unchecked_mut(*c) }.push(e.clone(), p.muted());
            }
          }
          HeadMatcher::Not(set) => {
            // e.g. the head matcher is `Not(['a', 'b'])`, the `set` is `['a', 'b']`
            known_map.for_each_entry_mut(|c, vec| {
              // e.g. if the head char is `'c'` which is not in `set`, add the action to the vec
              if !set.contains(&c) {
                vec.push(e.clone(), p.muted());
              }
            });
            unknown_fallback.push(e.clone(), p.muted());
          }
          HeadMatcher::Unknown => {
            unknown_fallback.push(e.clone(), p.muted());
          }
        }
      } else {
        // no head matcher, add the action to all known chars
        known_map.for_each_entry_mut(|_, vec| {
          vec.push(e.clone(), p.muted());
        });
        // and unknown fallback
        unknown_fallback.push(e.clone(), p.muted());
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    Self {
      known_map: known_map.build(),
      unknown_fallback,
    }
  }

  /// Get actions by the next char.
  #[inline]
  pub fn get(&self, next: char) -> &RuntimeActions<Kind, State, Heap> {
    self
      .known_map
      .get(next as usize)
      .unwrap_or(&self.unknown_fallback)
  }
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use crate::lexer::{
//     action::{exact, regex, Action},
//     token::MockKind,
//   };

//   fn push_helper<K, S>(actions: &mut RuntimeActions<K, S, ()>, a: Action<K, S, ()>) {
//     let (exec, options) = a.into_rc();
//     actions.push(exec, options.muted());
//   }

//   #[test]
//   fn test_head_map_actions() {
//     let mut actions: RuntimeActions<MockKind<()>, i32, ()> = RuntimeActions::new();
//     assert_eq!(actions.len(), 0);

//     push_helper(&mut actions, exact("a"));
//     assert_eq!(actions.len(), 1);
//     assert_eq!(actions.immutables.len(), 1);

//     push_helper(&mut actions, exact("b"));
//     assert_eq!(actions.len(), 2);
//     assert_eq!(actions.immutables.len(), 2);

//     push_helper(&mut actions, exact("c").prepare(|input| *input.state += 1));
//     assert_eq!(actions.len(), 3);
//     assert_eq!(actions.immutables.len(), 2);
//     assert_eq!(actions.rest.len(), 1);

//     push_helper(&mut actions, exact("d"));
//     assert_eq!(actions.len(), 4);
//     assert_eq!(actions.immutables.len(), 2);
//     assert_eq!(actions.rest.len(), 2);

//     push_helper(&mut actions, exact("e").prepare(|input| *input.state += 1));
//     assert_eq!(actions.len(), 5);
//     assert_eq!(actions.immutables.len(), 2);
//     assert_eq!(actions.rest.len(), 3);
//   }

//   fn assert_immutable_actions_eq(
//     actions: &RuntimeActions<MockKind<()>, (), ()>,
//     expected: Vec<Action<MockKind<()>, (), ()>>,
//   ) {
//     assert_eq!(actions.len(), expected.len());
//     for i in 0..actions.immutables.len() {
//       assert_eq!(actions.immutables.muted()[i], expected[i].muted());
//     }
//   }

//   #[test]
//   fn test_head_map() {
//     let (execs, props) = vec![
//       exact("a"),
//       exact("aa"),
//       exact("b"),
//       regex("[^c]").unchecked_head_not(['c']),
//       regex(".").unchecked_head_unknown(),
//       regex("a_muted").unchecked_head_in(['a']).mute(),
//       regex("no_head"),
//     ]
//     .into_iter()
//     .map(|a| a.into_rc())
//     .unzip();

//     let hm = HeadMap::new(&execs, &props, HeadMap::collect_all_known(&props));

//     // collect all known heads
//     assert!(hm.known_map.contains_key(&'a'));
//     assert!(hm.known_map.contains_key(&'b'));
//     assert!(hm.known_map.contains_key(&'c'));
//     assert_eq!(hm.known_map.len(), 3);

//     // check actions
//     assert_immutable_actions_eq(
//       &hm.get('a'),
//       vec![
//         exact("a"),
//         exact("aa"),
//         regex("[^c]").unchecked_head_not(['c']),
//         regex("a_muted").unchecked_head_in(['a']).mute(),
//         regex("no_head"),
//       ],
//     );
//     assert_immutable_actions_eq(
//       &hm.get('b'),
//       vec![
//         exact("b"),
//         regex("[^c]").unchecked_head_not(['c']),
//         regex("no_head"),
//       ],
//     );
//     assert_immutable_actions_eq(&hm.get('c'), vec![regex("no_head")]);
//     assert_immutable_actions_eq(
//       &hm.get('z'),
//       vec![
//         regex("[^c]").unchecked_head_not(['c']),
//         regex(".").unchecked_head_unknown(),
//         regex("no_head"),
//       ],
//     );
//   }
// }
