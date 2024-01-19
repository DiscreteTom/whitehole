use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse, Data, DeriveInput, Fields};

#[proc_macro_derive(TokenKind)]
pub fn token_kind_macro_derive(input: TokenStream) -> TokenStream {
  let ast: DeriveInput = parse(input).unwrap();

  // ensure derive is only used on enums
  let variants = match ast.data {
    Data::Enum(data) => data,
    _ => panic!("TokenKind can only be derived for enums"),
  }
  .variants;

  let name = &ast.ident;
  let match_content: Vec<proc_macro2::TokenStream> = variants
    .iter()
    .enumerate()
    .map(|(index, variant)| {
      let ident = &variant.ident;
      match variant.fields {
        Fields::Named(_) => {
          quote! {
            #name::#ident { .. } => #index,
          }
        }
        Fields::Unnamed(_) => {
          quote! {
            #name::#ident(..) => #index,
          }
        }
        Fields::Unit => {
          quote! {
            #name::#ident => #index,
          }
        }
      }
    })
    .collect();

  let gen = quote! {
    impl TokenKind for #name {
      fn id(&self) -> TokenKindId {
        match self {
          #(#match_content)*
        }
      }
    }
  };
  gen.into()
}
