use proc_macro::TokenStream;
use proc_macro2::Span;
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
/// enum MyKind { A, B(B), C(C) }
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
/// Besides, if the token kind derive `Default`:
/// ```
/// use whitehole_macros::token_kind;
/// #[token_kind]
/// #[derive(Default)]
/// enum MyKind {
///   #[default]
///   A,
///   B(i32),
///   C { c: i32 }
/// }
/// ```
/// the macro will also generate:
/// ```no_run
/// impl DefaultTokenKindIdBinding<MyKind> for MyKind { ... }
/// ```
#[proc_macro_attribute]
pub fn token_kind(_attr: TokenStream, input: TokenStream) -> TokenStream {
  common(quote! { whitehole }, input).into()
}

/// This is only used internally in whitehole.
#[proc_macro_attribute]
pub fn _token_kind(_attr: TokenStream,input: TokenStream) -> TokenStream {
  common(quote! { crate }, input).into()
}

/// Print the generated code for debugging.
/// This is only used internally in whitehole.
#[proc_macro_attribute]
pub fn _debug_token_kind(_attr: TokenStream,input: TokenStream) -> TokenStream {
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
  let vis = &ast.vis;
  let attrs = &ast.attrs;

  let mut gen = Vec::new();

  // override the original enum
  let generated_fields: Vec<_> = variants.iter().map(|variant| {
    let variant_name = &variant.ident;
    let variant_attrs = &variant.attrs;
    if matches!(variant.fields, Fields::Unit) {
      // for unit variants, we don't need to wrap them in unnamed fields.
      // with this design, we can make `#[derive(Default)]` and `#[default]` working
      // because `#[default]` only works for unit fields.
      quote! { #(#variant_attrs)* #variant_name, }
    } else {
      quote! { #(#variant_attrs)* #variant_name(#variant_name), }
    }
  }).collect();
  gen.push(quote! { #(#attrs)* #vis enum #enum_name { #(#generated_fields)* } });

  // generate a struct for each variant
  variants.iter().enumerate().for_each(|(index, variant)| {
    let variant_name = &variant.ident;
    let variant_attrs = &variant.attrs;

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
          #(#attrs)* pub struct #variant_name{ #(#generated_fields),* }
          impl Into<#enum_name> for #variant_name {
            fn into(self) -> #enum_name {
              #enum_name::#variant_name(self)
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
        gen.push(quote! {
          #(#attrs)* pub struct #variant_name(#(#generated_fields),*);
          impl Into<#enum_name> for #variant_name {
            fn into(self) -> #enum_name {
              #enum_name::#variant_name(self)
            }
          }
        });
      }
      Fields::Unit => {
        gen.push(quote! {
          #(#attrs)* pub struct #variant_name;
          impl Into<#enum_name> for #variant_name {
            fn into(self) -> #enum_name {
              #enum_name::#variant_name
            }
          }
        });
      }
    }

    gen.push(quote! {
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
    let mod_name = syn::Ident::new(&format!("_impl_sub_token_kind_{}", index), Span::call_site());
    let token_kind_id_const = syn::Ident::new(&format!("_TOKEN_KIND_ID_{}", index), Span::call_site());
    gen.push(quote! {
      mod #mod_name {
        use super::*;
        const #token_kind_id_const: #crate_name::lexer::token::TokenKindId<#crate_name::lexer::token::TokenKindIdBinding<#enum_name>> = #crate_name::lexer::token::TokenKindId::new(#index);
        impl #crate_name::lexer::token::SubTokenKind<#crate_name::lexer::token::TokenKindIdBinding<#enum_name>> for #variant_name {
          fn kind_id() -> 
            &'static #crate_name::lexer::token::TokenKindId<
              #crate_name::lexer::token::TokenKindIdBinding<#enum_name>
            >
          {
            &#token_kind_id_const
          }
        }
      }
    });

    // if a variant is the default variant, we will impl DefaultTokenKindIdBinding for it
    if variant_attrs.iter().any(|attr| attr.path.is_ident("default")) {
      gen.push(quote! {
        impl #crate_name::lexer::token::DefaultTokenKindIdBinding<#enum_name> for #enum_name {
          fn default_binding_kind_id() -> &'static #crate_name::lexer::token::TokenKindId<
            #crate_name::lexer::token::TokenKindIdBinding<#enum_name>
          > {
            #variant_name::kind_id()
          }
        }
      });
    }
  });

  quote! {
    #(#gen)*
  }
}
