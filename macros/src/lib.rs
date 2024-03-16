use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{self, parse, Data, DeriveInput, Fields};

// TODO: impl TokenKindGroup

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
/// impl TokenKind<MyKind> for A { ... }
///
/// pub struct B(pub i32);
/// impl Into<TokenKindIdBinding<MyKind>> for B { ... }
/// impl TokenKind<MyKind> for B { ... }
///
/// pub struct C { pub c: i32 }
/// impl Into<TokenKindIdBinding<MyKind>> for C { ... }
/// impl TokenKind<MyKind> for C { ... }
///
/// impl TokenKind<MyKind> for MyKind { ... }
/// ```
#[proc_macro_derive(TokenKind, attributes(TokenKindGroup))]
pub fn token_kind_macro_derive(input: TokenStream) -> TokenStream {
  common(quote! { whitehole }, input)
}

/// This is used internally in whitehole.
#[proc_macro_derive(_TokenKind, attributes(TokenKindGroup))]
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

    // impl TokenKind for the generated struct
    gen.push(quote! {
      impl #crate_name::lexer::token::TokenKind<#enum_name> for #variant_name {
        fn possible_kinds() -> std::collections::HashSet<
          #crate_name::lexer::token::TokenKindId<#enum_name>
        >
        {
          // the generated struct only has one kind
          std::collections::HashSet::from([#crate_name::lexer::token::TokenKindId::new(#index)])
        }
      }
    });

    // TODO: collect groups and generated structs
  });

  // impl TokenKind for the enum
  let generated_token_kind_ids: Vec<_> = (0..variants.len())
    .into_iter()
    .map(|i| {
      quote! { #crate_name::lexer::token::TokenKindId::new(#i) }
    })
    .collect();
  gen.push(quote! {
    impl #crate_name::lexer::token::TokenKind<#enum_name> for #enum_name {
      fn possible_kinds() -> std::collections::HashSet<
        #crate_name::lexer::token::TokenKindId<#enum_name>
      >
      {
        std::collections::HashSet::from([#(#generated_token_kind_ids),*])
      }
    }
  });

  let res = quote! {
    #(#gen)*
  };
  // println!("{}", gen.to_string());
  res.into()
}
