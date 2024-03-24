use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{self, parse, Data, DeriveInput, Fields};

/// This macro will transform an enum into a token kind.
/// # Examples
/// The following code
/// ```
/// use whitehole_macros::token_kind;
/// #[token_kind]
/// enum MyKind {
///   A,
///   B(i32),
///   C { c: i32 }
/// }
/// ```
/// will be transformed into:
/// ```no_run
/// enum MyKind { A(A), B(B), C(C) }
/// pub struct A;
/// impl Into<MyKind> for A { ... }
/// impl Into<TokenKindIdBinding<MyKind>> for A { ... }
/// impl SubTokenKind<TokenKindIdBinding<MyKind>> for A { ... }
///
/// pub struct B(pub i32);
/// impl Into<MyKind> for B { ... }
/// impl Into<TokenKindIdBinding<MyKind>> for B { ... }
/// impl SubTokenKind<TokenKindIdBinding<MyKind>> for B { ... }
///
/// pub struct C { pub c: i32 }
/// impl Into<MyKind> for C { ... }
/// impl Into<TokenKindIdBinding<MyKind>> for C { ... }
/// impl SubTokenKind<TokenKindIdBinding<MyKind>> for C { ... }
/// ```
#[proc_macro_attribute]
pub fn token_kind(_attr: TokenStream, input: TokenStream) -> TokenStream {
  common(quote! { whitehole }, input).into()
}

// TODO: make this only available in dev mode for whitehole
/// This is only used internally in whitehole.
#[proc_macro_attribute]
pub fn _token_kind(_attr: TokenStream,input: TokenStream) -> TokenStream {
  common(quote! { crate }, input).into()
}

// TODO: make this only available in dev mode for whitehole
/// This is only used internally in whitehole.
#[proc_macro_attribute]
pub fn debug_token_kind(_attr: TokenStream,input: TokenStream) -> TokenStream {
  let ts = common(quote! { crate }, input);
  println!("{}", ts.to_string());
  ts.into()
}

fn common(crate_name: proc_macro2::TokenStream, input: TokenStream) -> proc_macro2::TokenStream {
  let ast: DeriveInput = parse(input).unwrap();

  // ensure the macro is only used on enums, then retrieve variants
  let variants = match ast.data {
    Data::Enum(data) => data,
    _ => panic!("this macro can only be applied for enums"),
  }
  .variants;
  let enum_name = &ast.ident;

  let mut gen = Vec::new();

  // override the original enum
  // TODO: what about other derive macros?
  let generated_fields: Vec<_> = variants.iter().map(| variant| {
    let variant_name = &variant.ident;
    quote! { #variant_name(#variant_name), }
  }).collect();
  let vis = &ast.vis;
  let attrs = &ast.attrs;
  gen.push(quote! { #(#attrs)* #vis enum #enum_name { #(#generated_fields)* } });

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
        gen.push(quote! {
          pub struct #variant_name{ #(#generated_fields),* }
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
        gen.push(quote! {
          pub struct #variant_name(#(#generated_fields),*);
        });
      }
      Fields::Unit => {
        gen.push(quote! {
          pub struct #variant_name;
        });
      }
    }

    gen.push(quote! {
      impl Into<#enum_name> for #variant_name {
        fn into(self) -> #enum_name {
          #enum_name::#variant_name(self)
        }
      }
      impl Into<#crate_name::lexer::token::TokenKindIdBinding<#enum_name>> for #variant_name {
        fn into(self) -> #crate_name::lexer::token::TokenKindIdBinding<#enum_name> {
          #crate_name::lexer::token::TokenKindIdBinding::new(self)
        }
      }
    });

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
