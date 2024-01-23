use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse, Data, DeriveInput, Fields};

#[proc_macro_derive(TokenKind)]
pub fn token_kind_macro_derive(input: TokenStream) -> TokenStream {
  let ast: DeriveInput = parse(input).unwrap();

  // ensure derive is only used on enums, then retrieve variants
  let variants = match ast.data {
    Data::Enum(data) => data,
    _ => panic!("TokenKind can only be derived for enums"),
  }
  .variants;

  let crate_name = if std::env::var("CARGO_CRATE_NAME").unwrap() == "whitehole" {
    quote! { crate }
  } else {
    quote! { whitehole }
  };
  let enum_name = &ast.ident;
  let match_arms: Vec<proc_macro2::TokenStream> = variants
    .iter()
    .enumerate()
    .map(|(index, variant)| {
      let variant_name = &variant.ident;
      match variant.fields {
        Fields::Named(_) => {
          quote! {
            #enum_name::#variant_name { .. } => #index,
          }
        }
        Fields::Unnamed(_) => {
          quote! {
            #enum_name::#variant_name(..) => #index,
          }
        }
        Fields::Unit => {
          quote! {
            #enum_name::#variant_name => #index,
          }
        }
      }
    })
    .collect();

  let gen = quote! {
    impl #crate_name::lexer::token::TokenKind for #enum_name {
      fn id(&self) -> #crate_name::lexer::token::TokenKindId {
        match self {
          #(#match_arms)*
        }
      }
    }
  };
  gen.into()
}
