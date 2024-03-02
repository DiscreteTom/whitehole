use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse, Data, DeriveInput, Fields};

#[proc_macro_derive(TokenKind)]
pub fn token_kind_macro_derive(input: TokenStream) -> TokenStream {
  common(quote! { whitehole }, input)
}

/// This is used internally in whitehole.
#[proc_macro_derive(_TokenKind)]
pub fn internal_token_kind_macro_derive(input: TokenStream) -> TokenStream {
  common(quote! { crate }, input)
}

fn common(crate_name: proc_macro2::TokenStream, input: TokenStream) -> TokenStream {
  let ast: DeriveInput = parse(input).unwrap();

  // ensure derive is only used on enums, then retrieve variants
  let variants = match ast.data {
    Data::Enum(data) => data,
    _ => panic!("TokenKind can only be derived for enums"),
  }
  .variants;

  let enum_name = &ast.ident;
  let match_arms: Vec<proc_macro2::TokenStream> = variants
    .iter()
    .enumerate()
    .map(|(index, variant)| {
      let variant_name = &variant.ident;
      match variant.fields {
        Fields::Named(_) => {
          quote! {
            #enum_name::#variant_name { .. } => #crate_name::lexer::token::TokenKindId::new(#index),
          }
        }
        Fields::Unnamed(_) => {
          quote! {
            #enum_name::#variant_name(..) => #crate_name::lexer::token::TokenKindId::new(#index),
          }
        }
        Fields::Unit => {
          quote! {
            #enum_name::#variant_name => #crate_name::lexer::token::TokenKindId::new(#index),
          }
        }
      }
    })
    .collect();

  let gen = quote! {
    impl #crate_name::lexer::token::TokenKind<#enum_name> for #enum_name {
      fn id(&self) -> #crate_name::lexer::token::TokenKindId<#enum_name> {
        match self {
          #(#match_arms)*
        }
      }
    }
  };
  gen.into()
}
