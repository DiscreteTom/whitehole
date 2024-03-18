use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{self, parse, Data, DeriveInput, Fields};

/// This derive macro will transform an enum into a token kind.
/// # Examples
/// The following code
/// ```
/// use whitehole_macros::TokenKind;
/// #[derive(TokenKind)]
/// enum MyKind {
///   A,
///   B(i32),
///   C { c: i32 }
/// }
/// ```
/// will generate:
/// ```no_run
/// pub struct A;
/// impl Into<TokenKindIdBinding<MyKind>> for A { ... }
/// impl SubTokenKind<TokenKindIdBinding<MyKind>> for A { ... }
///
/// pub struct B(pub i32);
/// impl Into<TokenKindIdBinding<MyKind>> for B { ... }
/// impl SubTokenKind<TokenKindIdBinding<MyKind>> for B { ... }
///
/// pub struct C { pub c: i32 }
/// impl Into<TokenKindIdBinding<MyKind>> for C { ... }
/// impl SubTokenKind<TokenKindIdBinding<MyKind>> for C { ... }
/// ```
#[proc_macro_derive(TokenKind)]
pub fn token_kind_macro_derive(input: TokenStream) -> TokenStream {
  common(quote! { whitehole }, input).into()
}

// TODO: make this only available in dev mode for whitehole
/// This is only used internally in whitehole.
#[proc_macro_derive(_TokenKind)]
pub fn internal_token_kind_macro_derive(input: TokenStream) -> TokenStream {
  common(quote! { crate }, input).into()
}

// TODO: make this only available in dev mode for whitehole
/// This is only used internally in whitehole.
#[proc_macro_derive(__TokenKind)]
pub fn debug_token_kind_macro_derive(input: TokenStream) -> TokenStream {
  let ts = common(quote! { crate }, input);
  println!("{}", ts.to_string());
  ts.into()
}

fn common(crate_name: proc_macro2::TokenStream, input: TokenStream) -> proc_macro2::TokenStream {
  let ast: DeriveInput = parse(input).unwrap();

  // ensure derive is only used on enums, then retrieve variants
  let variants = match ast.data {
    Data::Enum(data) => data,
    _ => panic!("TokenKind can only be derived for enums"),
  }
  .variants;
  let enum_name = &ast.ident;

  let mut gen = Vec::new();
  variants.iter().enumerate().for_each(|(index, variant)| {
    let variant_name = &variant.ident;

    // generate a struct for each variant
    match &variant.fields {
      Fields::Named(fields) => {
        let generated_fields: Vec<_> = fields
          .named
          .iter()
          .map(|f| {
            let ts = f.to_token_stream();
            quote! { pub #ts } // make all fields public
          })
          .collect();
        let generated_assign: Vec<_> = fields
          .named
          .iter()
          .map(|f| {
            let name = f.ident.clone().unwrap();
            quote! { #name: self.#name }
          })
          .collect();
        gen.push(quote! {
          pub struct #variant_name{ #(#generated_fields),* }
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
        let generated_fields: Vec<_> = fields
          .unnamed
          .iter()
          .map(|f| {
            let ts = f.ty.to_token_stream();
            quote! { pub #ts } // make all fields public
          })
          .collect();
        let generated_assign: Vec<_> = (0..fields.unnamed.len())
          .into_iter()
          .map(|i| {
            let i = syn::Index::from(i);
            quote! { self.#i }
          })
          .collect();
        gen.push(quote! {
          pub struct #variant_name(#(#generated_fields),*);
          impl Into<#crate_name::lexer::token::TokenKindIdBinding<#enum_name>> for #variant_name {
            fn into(self) -> #crate_name::lexer::token::TokenKindIdBinding<#enum_name> {
              #crate_name::lexer::token::TokenKindIdBinding::new(
                #index,
                #enum_name::#variant_name(#(#generated_assign),*)
              )
            }
          }
        });
      }
      Fields::Unit => {
        gen.push(quote! {
          pub struct #variant_name;
          impl Into<#crate_name::lexer::token::TokenKindIdBinding<#enum_name>> for #variant_name {
            fn into(self) -> #crate_name::lexer::token::TokenKindIdBinding<#enum_name> {
              #crate_name::lexer::token::TokenKindIdBinding::new(#index, #enum_name::#variant_name)
            }
          }
        });
      }
    }

    // impl SubTokenKind for the generated struct
    // we should impl SubTokenKind for `TokenKindIdBinding<MyKind>` instead of `MyKind`
    // because for `Action<TokenKindIdBinding<MyKind>>` the `action.kind_id` should be
    // `TokenKindId<TokenKindIdBinding<MyKind>>` instead of `TokenKindId<MyKind>`
    gen.push(quote! {
      impl #crate_name::lexer::token::SubTokenKind<#crate_name::lexer::token::TokenKindIdBinding<#enum_name>> for #variant_name {
        fn kind_id() -> 
          #crate_name::lexer::token::TokenKindId<
            #crate_name::lexer::token::TokenKindIdBinding<#enum_name>
          >
        {
          #crate_name::lexer::token::TokenKindId::new(#index)
        }
      }
    });

    // TODO: collect groups and generated structs
  });

  quote! {
    #(#gen)*
  }
}
