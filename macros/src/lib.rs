use proc_macro::TokenStream;
use quote::{quote, ToTokens};
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
        // TODO: is pattern matching the best way to do this?
        // e.g. maybe mem::Discriminant is faster? need benchmarks
        // see https://doc.rust-lang.org/std/mem/fn.discriminant.html
        match self {
          #(#match_arms)*
        }
      }
    }
  };
  gen.into()
}

#[proc_macro_derive(NewTokenKind, attributes(token_kind))]
pub fn new_token_kind_macro_derive(input: TokenStream) -> TokenStream {
  new_common(quote! { whitehole }, input)
}

/// This is used internally in whitehole.
#[proc_macro_derive(_NewTokenKind, attributes(token_kind))]
pub fn new_internal_token_kind_macro_derive(input: TokenStream) -> TokenStream {
  new_common(quote! { crate }, input)
}

fn new_common(crate_name: proc_macro2::TokenStream, input: TokenStream) -> TokenStream {
  let ast: DeriveInput = parse(input).unwrap();

  // ensure derive is only used on enums, then retrieve variants
  let variants = match ast.data {
    Data::Enum(data) => data,
    _ => panic!("TokenKind can only be derived for enums"),
  }
  .variants;

  let enum_name = &ast.ident;
  let mut generated_vec: Vec<proc_macro2::TokenStream> = Vec::new();
  variants
    .iter()
    .enumerate()
    .for_each(|(index, variant)| {
      let variant_name = &variant.ident;
      match &variant.fields {
        Fields::Named(fields) => {
          let generated_fields: Vec<_> = fields.named.iter().map(|f|{
            let ts = f.to_token_stream(); 
            quote! { pub #ts } // make all fields public
          }).collect();
          let generated_assign: Vec<_> = fields.named.iter().map(|f| {
            let name = f.ident.clone().unwrap();
            quote!{ #name: self.#name }
          }).collect();
          generated_vec.push(quote! {
            pub struct #variant_name{ #(#generated_fields),* };
            impl Into<#crate_name::lexer::token::TokenKindIdBinding<#enum_name>> for #variant_name {
              fn into(self) -> #crate_name::lexer::token::TokenKindIdBinding<#enum_name> {
                #crate_name::lexer::token::TokenKindIdBinding::new(
                  #index, 
                  #enum_name::#variant_name{ #(#generated_assign),* }
                )
              }
            }
          });
        }
        Fields::Unnamed(fields) => {
          let types: Vec<_> = fields.unnamed.iter().map(|f|{
            let ts = f.ty.to_token_stream(); 
            quote!{ pub #ts } // make all fields public
          }).collect();
          let placeholders: Vec<_> = (0..fields.unnamed.len()).into_iter().map(|i|{
            let i = syn::Index::from(i);
             quote!{ self.#i }
          }).collect();
          generated_vec.push(quote! {
            pub struct #variant_name(#(#types),*);
            impl Into<#crate_name::lexer::token::TokenKindIdBinding<#enum_name>> for #variant_name {
              fn into(self) -> #crate_name::lexer::token::TokenKindIdBinding<#enum_name> {
                #crate_name::lexer::token::TokenKindIdBinding::new(
                  #index, 
                  #enum_name::#variant_name(#(#placeholders),*)
                )
              }
            }
          });
        }
        Fields::Unit => {
          generated_vec.push(quote! {
            pub struct #variant_name;
            impl Into<#crate_name::lexer::token::TokenKindIdBinding<#enum_name>> for #variant_name {
              fn into(self) -> #crate_name::lexer::token::TokenKindIdBinding<#enum_name> {
                #crate_name::lexer::token::TokenKindIdBinding::new(#index, #enum_name::#variant_name)
              }
            }
          });
        }
      }
      generated_vec.push(quote! {
        impl #variant_name {
          fn possible_kinds() -> (
            std::collections::HashSet<#crate_name::lexer::token::TokenKindId<#crate_name::lexer::token::TokenKindIdBinding<#enum_name>>>,
            std::marker::PhantomData<#variant_name>
          ) {
            (
              std::collections::HashSet::from([#crate_name::lexer::token::TokenKindId::new(#index)]),
              std::marker::PhantomData
            )
          }
        }
      });
    });

  let gen = quote! {
    #(#generated_vec)*
  };
  // println!("{}", gen.to_string());
  gen.into()
}
