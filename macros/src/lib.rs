use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{self, parse, Data, DeriveInput, Fields, LitStr};

/// This macro will transform an enum into a kind.
/// # Examples
/// The following code
/// ```
/// use whitehole::kind::whitehole_kind;
/// #[whitehole_kind]
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
/// impl SubKind for A { ... }
///
/// pub struct B(pub i32);
/// impl Into<MyKind> for B { ... }
/// impl SubKind for B { ... }
///
/// pub struct C { pub c: i32 }
/// impl Into<MyKind> for C { ... }
/// impl SubKind for C { ... }
/// ```
/// Besides, if the kind derive [`Default`]:
/// ```
/// use whitehole::kind::whitehole_kind;
/// #[whitehole_kind]
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
/// impl DefaultSubKind for MyKind { ... }
/// ```
/// # Limitations
/// Generics are not supported yet.
#[proc_macro_attribute]
pub fn whitehole_kind(_attr: TokenStream, input: TokenStream) -> TokenStream {
  common(quote! { whitehole }, input).into()
}

/// Same as [`whitehole_kind()`].
/// This is only used internally in whitehole.
#[proc_macro_attribute]
pub fn _whitehole_kind(_attr: TokenStream, input: TokenStream) -> TokenStream {
  common(quote! { crate }, input).into()
}

/// Print the generated code for debugging.
/// This is only used internally in whitehole.
#[proc_macro_attribute]
pub fn _debug_whitehole_kind(_attr: TokenStream, input: TokenStream) -> TokenStream {
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
    // this is required by `KindIdBinding::new`
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

    // impl SubKind for the generated struct
    gen.push(quote! {
      // impl SubKind so users can get the kind id from the type instead of the value
      impl #crate_name::kind::SubKind for #variant_name {
        type Kind = #enum_name;
        const VARIANT_INDEX: usize = #index;
      }
    });

    // if a variant is the default variant, we will impl DefaultKindId for the enum
    if variant_attrs
      .iter()
      .any(|attr| attr.path.is_ident("default"))
    {
      gen.push(quote! {
        impl #crate_name::kind::DefaultSubKind for #enum_name {
          type Default = #variant_name;
        }
      });
    }
  });

  quote! {
    #(#gen)*
  }
}

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
