use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{self, parse, Data, DeriveInput, Fields, LitStr};

/// Generate a function that checks if a character is in the provided literal string
/// using the [`matches!`] macro.
/// This is usually faster than using [`str::contains`].
/// # Examples
/// ```
/// use whitehole::in_str;
/// let _ = in_str!("abc");
/// // equals to
/// let _ = |c: char| matches!(c, 'a' | 'b' | 'c');
/// // usually faster than
/// let _ = |c: char| "abc".contains(c);
///
/// // escape will be handled automatically
/// let _ = in_str!("\n\u{10ffff}");
/// // equals to
/// let _ = |c: char| matches!(c, '\n' | '\u{10ffff}');
/// ```
#[proc_macro]
pub fn in_str(item: TokenStream) -> TokenStream {
  let mut gen = Vec::new();
  let s = parse::<LitStr>(item).unwrap().value();
  for c in s.chars() {
    gen.push(quote! { #c });
  }
  quote! { |c: char| matches!(c, #(#gen)|*) }.into()
}
