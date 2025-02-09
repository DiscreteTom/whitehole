use whitehole::{
  action::Action,
  combinator::{eat, next, recur},
  parser::Parser,
};

// see https://semver.org/#backusnaur-form-grammar-for-valid-semver-versions
fn build_entry() -> impl Action {
  let letter = || next(|c| c.is_ascii_alphabetic());
  let positive_digit = || next(|c| matches!(c, '1'..'9'));
  let digit = || next(|c| c.is_ascii_digit());
  let digits = || digit() * (1..);
  let non_digit = || letter() | '-';
  let identifier_character = || digit() | non_digit();
  let identifier_characters = || identifier_character() * (1..);
  let numeric_identifier = || eat('0') | positive_digit() + digits().optional();

  let alphanumeric_identifier = || {
    (non_digit() + identifier_characters().optional())
      | (identifier_characters() + non_digit() + identifier_characters().optional())
  };

  let build_identifier = || alphanumeric_identifier() | digits();
  let pre_release_identifier = || alphanumeric_identifier() | numeric_identifier();

  let (dot_separated_build_identifiers, setter) = recur::<_, (), (), ()>();
  setter.boxed(build_identifier() + (eat('.') + dot_separated_build_identifiers()).optional());

  let build = || dot_separated_build_identifiers();

  let (dot_separated_pre_release_identifiers, setter) = recur::<_, (), (), ()>();
  setter.boxed(
    pre_release_identifier() + (eat('.') + dot_separated_pre_release_identifiers()).optional(),
  );

  let pre_release = || dot_separated_pre_release_identifiers();
  let patch = || numeric_identifier();
  let minor = || numeric_identifier();
  let major = || numeric_identifier();
  let version_core = || major() + eat('.') + minor() + eat('.') + patch();

  let valid_semver = || {
    version_core()
      + ((eat('-') + pre_release() + (eat('+') + build()).optional())
        | (eat('+') + build()).optional())
  };

  valid_semver()
}

fn helper(input: &str) -> bool {
  Parser::builder()
    .entry(build_entry())
    .build(input)
    .next()
    .is_some()
}

fn run_tests() {
  assert!(helper("1.0.0"));
  assert!(helper("2.1.0-alpha"));
  assert!(helper("0.3.7+build.42"));
  assert!(helper("1.2.3-beta.1+build.1"));
  assert!(!helper("1.2..3"));
  assert!(!helper("1.2.-3"));
  assert!(helper("1.2.3+meta-data"));
}

fn main() {
  run_tests();
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_semver() {
    run_tests();
  }
}
