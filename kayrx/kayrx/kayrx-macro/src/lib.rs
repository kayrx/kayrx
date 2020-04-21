//! language-level attributes for Kayrx.
//!
//! # Examples
//!
//! ```
//! #[kayrx::main]
//! async fn main() {
//!     println!("Hello, world!");
//! }
//! ```
//!

#![recursion_limit = "512"]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// Enables an async main function.
///
/// # Examples
///
/// ```ignore
/// #[kayrx::main]
/// async fn main() -> std::io::Result<()> {
///     Ok(())
/// }
/// ```
#[cfg(not(test))] // NOTE: exporting main breaks tests,  Work around for rust-lang/rust#62127
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let ret = &input.sig.output;
    let inputs = &input.sig.inputs;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    if name != "main" {
        return TokenStream::from(quote_spanned! { name.span() =>
            compile_error!("only the main function can be tagged with #[kayrx::main]"),
        });
    }

    if input.sig.asyncness.is_none() {
        return TokenStream::from(quote_spanned! { input.span() =>
            compile_error!("the async keyword is missing from the function declaration"),
        });
    }

    let result = quote! {
        fn main() #ret {
            #(#attrs)*
            async fn main(#inputs) #ret {
                #body
            }

            kayrx::karx::exec(async {
                main().await
            })
        }

    };

    result.into()
}

/// Enables an async test function.
///
/// # Examples
///
/// ```ignore
/// #[kayrx::test]
/// async fn my_test() -> std::io::Result<()> {
///     assert_eq!(2 * 2, 4);
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    if input.sig.asyncness.is_none() {
        return TokenStream::from(quote_spanned! { input.span() =>
            compile_error!("the async keyword is missing from the function declaration"),
        });
    }

    let result = quote! {
        #[test]
        #(#attrs)*
        fn #name() #ret {
            kayrx::karx::exec(async { #body })
        }
    };

    result.into()
}


/// Enables an futures async main function.
///
/// # Examples
///
/// ```ignore
/// #[kayrx::futures]
/// async fn main() -> std::io::Result<()> {
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn futures(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let ret = &input.sig.output;
    let inputs = &input.sig.inputs;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    if name != "main" {
        return TokenStream::from(quote_spanned! { name.span() =>
            compile_error!("only the main function can be tagged with #[kayrx::futures]"),
        });
    }

    if input.sig.asyncness.is_none() {
        return TokenStream::from(quote_spanned! { input.span() =>
            compile_error!("the futures async keyword is missing from the function declaration"),
        });
    }

    let result = quote! {
        fn main() #ret {
            #(#attrs)*
            async fn main(#inputs) #ret {
                #body
            }

            kayrx::karx::futures::default().exec(async {
                main().await
            })
        }

    };

    result.into()
}