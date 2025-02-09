use whitehole::{
  action::Action,
  combinator::{eat, next},
  parser::Parser,
};

// see https://semver.org/#backusnaur-form-grammar-for-valid-semver-versions
fn build_entry() -> impl Action {
  let letter = || next(|c| c.is_ascii_alphabetic());
  let positive_digit = || next(|c| matches!(c, '1'..='9'));
  let digit = || next(|c| c.is_ascii_digit());
  let digits = || digit() * (1..);
  let non_digit = || letter() | '-';
  let identifier_character = || digit() | non_digit();
  let identifier_characters = || identifier_character() * (1..);
  let numeric_identifier = || eat('0') | positive_digit() + digits().optional();

  let alphanumeric_identifier =
    || (non_digit() + identifier_characters().optional()) | (digit() + identifier_characters());

  let build_identifier = || alphanumeric_identifier() | digits();
  let pre_release_identifier = || alphanumeric_identifier() | numeric_identifier();

  let dot_separated_build_identifiers = || (build_identifier() * (1..)).sep('.');
  let build = || dot_separated_build_identifiers();
  let dot_separated_pre_release_identifiers = || (pre_release_identifier() * (1..)).sep('.');
  let pre_release = || dot_separated_pre_release_identifiers();
  let patch = || numeric_identifier();
  let minor = || numeric_identifier();
  let major = || numeric_identifier();
  let version_core = || major() + eat('.') + minor() + eat('.') + patch();

  let valid_semver = || {
    version_core()
      + ((eat('-') + pre_release() + (eat('+') + build()).optional())
        | (eat('+') + build()).optional())
      .optional()
  };

  valid_semver()
}

fn helper(input: &str) -> bool {
  let mut parser = Parser::builder().entry(build_entry()).build(input);
  let r = parser.next();

  let rest = parser.instant().rest();
  if !rest.is_empty() {
    panic!("Failed to parse '{}', remaining: {}", input, rest);
  }

  r.is_some()
}

fn accept(input: &str) {
  if !helper(input) {
    panic!("Failed to accept '{}'", input);
  }
}

fn reject(input: &str) {
  if helper(input) {
    panic!("Failed to reject '{}'", input);
  }
}

fn run_tests() {
  accept("0.0.4");
  accept("1.2.3");
  accept("10.20.30");
  accept("1.1.2-prerelease+meta");
  accept("1.1.2+meta");
  accept("1.1.2+meta-valid");
  accept("1.0.0-alpha");
  accept("1.0.0-beta");
  accept("1.0.0-alpha.beta");
  accept("1.0.0-alpha.beta.1");
  accept("1.0.0-alpha.1");
  accept("1.0.0-alpha0.valid");
  accept("1.0.0-alpha.0valid");
  accept("1.0.0-alpha-a.b-c-somethinglong+build.1-aef.1-its-okay");
  accept("1.0.0-rc.1+build.1");
  accept("2.0.0-rc.1+build.123");
  accept("1.2.3-beta");
  accept("10.2.3-DEV-SNAPSHOT");
  accept("1.2.3-SNAPSHOT-123");
  accept("1.0.0");
  accept("2.0.0");
  accept("1.1.7");
  accept("2.0.0+build.1848");
  accept("2.0.1-alpha.1227");
  accept("1.0.0-alpha+beta");
  accept("1.2.3----RC-SNAPSHOT.12.9.1--.12+788");
  accept("1.2.3----R-S.12.9.1--.12+meta");
  accept("1.2.3----RC-SNAPSHOT.12.9.1--.12");
  accept("1.0.0+0.build.1-rc.10000aaa-kk-0.1");
  accept("99999999999999999999999.999999999999999999.99999999999999999");
  accept("1.0.0-0A.is.legal");

  reject("1");
  reject("1.2");
  reject("1.2.3-0123");
  reject("1.2.3-0123.0123");
  reject("1.1.2+.123");
  reject("+invalid");
  reject("-invalid");
  reject("-invalid+invalid");
  reject("-invalid.01");
  reject("alpha");
  reject("alpha.beta");
  reject("alpha.beta.1");
  reject("alpha.1");
  reject("alpha+beta");
  reject("alpha_beta");
  reject("alpha.");
  reject("alpha..");
  reject("beta");
  reject("1.0.0-alpha_beta");
  reject("-alpha.");
  reject("1.0.0-alpha..");
  reject("1.0.0-alpha..1");
  reject("1.0.0-alpha...1");
  reject("1.0.0-alpha....1");
  reject("1.0.0-alpha.....1");
  reject("1.0.0-alpha......1");
  reject("1.0.0-alpha.......1");
  reject("01.1.1");
  reject("1.01.1");
  reject("1.1.01");
  reject("1.2");
  reject("1.2.3.DEV");
  reject("1.2-SNAPSHOT");
  reject("1.2.31.2.3----RC-SNAPSHOT.12.09.1--..12+788");
  reject("1.2-RC-SNAPSHOT");
  reject("-1.0.3-gamma+b7718");
  reject("+justmeta");
  reject("9.8.7+meta+meta");
  reject("9.8.7-whatever+meta+meta");
  reject("99999999999999999999999.999999999999999999.99999999999999999----RC-SNAPSHOT.12.09.1--------------------------------..12");
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
