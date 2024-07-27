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
/// pub enum MyKind {
///   A,
///   B(i32),
///   C { c: i32 }
/// }
/// ```
/// will be transformed into:
/// ```no_run
/// pub enum MyKind { A, B(B), C(C) }
/// pub struct A;
/// impl Into<MyKind> for A { ... }
/// impl Into<TokenKindIdBinding<MyKind>> for A { ... }
/// impl Into<&'static TokenKindId<TokenKindIdBinding<MyKind>>> for A { ... }
/// impl SubTokenKind for A { ... }
///
/// pub struct B(pub i32);
/// impl Into<MyKind> for B { ... }
/// impl Into<TokenKindIdBinding<MyKind>> for B { ... }
/// impl Into<&'static TokenKindId<TokenKindIdBinding<MyKind>>> for B { ... }
/// impl SubTokenKind for B { ... }
///
/// pub struct C { pub c: i32 }
/// impl Into<MyKind> for C { ... }
/// impl Into<TokenKindIdBinding<MyKind>> for C { ... }
/// impl Into<&'static TokenKindId<TokenKindIdBinding<MyKind>>> for C { ... }
/// impl SubTokenKind for C { ... }
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
/// # Limitations
/// Generics are not supported yet.
#[proc_macro_attribute]
pub fn token_kind(_attr: TokenStream, input: TokenStream) -> TokenStream {
  common(quote! { whitehole }, input).into()
}

/// Same as [`token_kind`].
/// This is only used internally in whitehole.
#[proc_macro_attribute]
pub fn _token_kind(_attr: TokenStream, input: TokenStream) -> TokenStream {
  common(quote! { crate }, input).into()
}

/// Print the generated code for debugging.
/// This is only used internally in whitehole.
#[proc_macro_attribute]
pub fn _debug_token_kind(_attr: TokenStream, input: TokenStream) -> TokenStream {
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
  let generated_fields: Vec<_> = variants
    .iter()
    .map(|variant| {
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
    })
    .collect();
  gen.push(quote! { #(#attrs)* #vis enum #enum_name { #(#generated_fields)* } });

  // generate a struct for each variant and implement traits for them
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
          #(#attrs)* #vis struct #variant_name{ #(#generated_fields),* } // no semicolon at the end
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
          #(#attrs)* #vis struct #variant_name(#(#generated_fields),*);
        });
      }
      Fields::Unit => {
        gen.push(quote! {
          #(#attrs)* #vis struct #variant_name;
        });
      }
    }

    // impl Into<MyKind> for the generated structs
    // this is required by `TokenKindIdBinding::new`
    match &variant.fields {
      Fields::Named(_) | Fields::Unnamed(_) => {
        gen.push(quote! {
          impl Into<#enum_name> for #variant_name {
            #[inline]
            fn into(self) -> #enum_name {
              #enum_name::#variant_name(self)
            }
          }
        });
      }
      Fields::Unit => {
        gen.push(quote! {
          impl Into<#enum_name> for #variant_name {
            #[inline]
            fn into(self) -> #enum_name {
              #enum_name::#variant_name
            }
          }
        });
      }
    }

    // impl Into<TokenKindIdBinding<MyKind>> for the generated structs
    // this is required by `Action::select`
    gen.push(quote! {
      impl Into<#crate_name::lexer::token::TokenKindIdBinding<#enum_name>> for #variant_name {
        #[inline]
        fn into(self) -> #crate_name::lexer::token::TokenKindIdBinding<#enum_name> {
          #crate_name::lexer::token::TokenKindIdBinding::new(self)
        }
      }
    });

    // impl SubTokenKind and Into<TokenKindId<TokenKindIdBinding<MyKind>>> for the generated struct
    // we should impl SubTokenKind for `TokenKindIdBinding<MyKind>` instead of `MyKind`
    // because for `Action<TokenKindIdBinding<MyKind>>` the `action.kind_id` should be
    // `TokenKindId<TokenKindIdBinding<MyKind>>` instead of `TokenKindId<MyKind>`
    let mod_name = syn::Ident::new(&format!("_impl_sub_token_kind_{}", index), Span::call_site());
    let token_kind_id_const = syn::Ident::new(&format!("_TOKEN_KIND_ID_{}", index), Span::call_site());
    let sub_token_kind_name = syn::LitStr::new(&variant_name.to_string(), Span::call_site());
    gen.push(quote! {
      // use a private mod to hide the constants
      mod #mod_name {
        use super::{#enum_name, #variant_name};
        use #crate_name::lexer::token::{SubTokenKind, TokenKindId, TokenKindIdBinding};
        const #token_kind_id_const: TokenKindId<TokenKindIdBinding<#enum_name>> = TokenKindId::new(#index, #sub_token_kind_name);
        // impl SubTokenKind so users can get the kind id from the type instead of the value
        impl SubTokenKind for #variant_name {
          type TokenKind = TokenKindIdBinding<#enum_name>;
          #[inline]
          fn kind_id() -> &'static TokenKindId<Self::TokenKind> {
            &#token_kind_id_const
          }
        }
        // impl Into<TokenKindId<TokenKindIdBinding<MyKind>>> for the generated struct
        // this is helpful in expectational lexing, if users wants to provide the expected kind id
        // they can just use the value (especially for unit variants)
        impl Into<&'static TokenKindId<TokenKindIdBinding<#enum_name>>> for #variant_name {
          #[inline]
          fn into(self) -> &'static TokenKindId<TokenKindIdBinding<#enum_name>> {
            &#token_kind_id_const
          }
        }
      }
    });

    // if a variant is the default variant, we will impl DefaultTokenKindIdBinding for the enum
    if variant_attrs.iter().any(|attr| attr.path.is_ident("default")) {
      gen.push(quote! {
        impl #crate_name::lexer::token::DefaultTokenKindIdBinding<#enum_name> for #enum_name {
          #[inline]
          fn default_kind_id() -> &'static #crate_name::lexer::token::TokenKindId<
            #crate_name::lexer::token::TokenKindIdBinding<#enum_name>
          > {
            <#variant_name as #crate_name::lexer::token::SubTokenKind>::kind_id()
          }
        }
      });
    }
  });

  quote! {
    #(#gen)*
  }
}
