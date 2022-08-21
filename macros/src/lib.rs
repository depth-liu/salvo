//! The macros lib of Savlo web server framework. Read more: <https://salvo.rs>
#![doc(html_favicon_url = "https://salvo.rs/favicon-32x32.png")]
#![doc(html_logo_url = "https://salvo.rs/images/logo.svg")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(private_in_public, unreachable_pub, unused_crate_dependencies)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, Item};

mod extract;
mod handler;
mod shared;

/// `handler` is a pro macro to help create `Handler` from function or impl block easily.
///
/// `Handler` is a trait, if `#[handler]` applied to `fn`,  `fn` will converted to a struct, and then implement `Handler`.
///
/// ```ignore
/// #[async_trait]
/// pub trait Handler: Send + Sync + 'static {
///     async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl);
/// }
/// ```
///
/// After use `handler`, you don't need to care arguments' order, omit unused arguments:
///
/// ```ignore
/// #[handler]
/// async fn hello_world() -> &'static str {
///     "Hello World"
/// }
/// ```
#[proc_macro_attribute]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let internal = shared::is_internal(args.iter());
    let item = parse_macro_input!(input as Item);
    match handler::generate(internal, item) {
        Ok(stream) => stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// `handler` is a pro macro to help create `Handler` from function easily.
///
/// Note: This is a deprecated version, please use `handler` instead, this will be removed in the future.
#[deprecated(
    since = "0.27.0",
    note = "please use `handler` instead, this will be removed in the future"
)]
#[proc_macro_attribute]
pub fn fn_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    handler(args, input)
}

/// Generate code for extractible type.
#[proc_macro_derive(Extractible, attributes(extract))]
pub fn derive_extractible(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as DeriveInput);
    match extract::generate(args) {
        Ok(stream) => stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse2;

    use super::*;

    #[test]
    fn test_handler_for_fn() {
        let input = quote! {
            #[handler]
            async fn hello(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
                res.render_plain_text("Hello World");
            }
        };
        let item = parse2(input).unwrap();
        assert_eq!(
            handler::generate(false, item).unwrap().to_string(),
            quote! {
                #[allow(non_camel_case_types)]
                #[derive(Debug)]
                struct hello;
                impl hello {
                    #[handler]
                    async fn hello(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
                        {
                            res.render_plain_text("Hello World");
                        }
                    }
                }
                #[salvo::async_trait]
                impl salvo::Handler for hello {
                    #[inline]
                    async fn handle(
                        &self,
                        req: &mut salvo::Request,
                        depot: &mut salvo::Depot,
                        res: &mut salvo::Response,
                        ctrl: &mut salvo::routing::FlowCtrl
                    ) {
                        Self::hello(req, depot, res, ctrl).await
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn test_handler_for_impl() {
        let input = quote! {
            #[handler]
            impl Hello {
                fn handle(req: &mut Request, depot: &mut Depot, res: &mut Response) {
                    res.render_plain_text("Hello World");
                }
            }
        };
        let item = parse2(input).unwrap();
        assert_eq!(
            handler::generate(false, item).unwrap().to_string(),
            quote! {
                #[handler]
                impl Hello {
                    fn handle(req: &mut Request, depot: &mut Depot, res: &mut Response) {
                        res.render_plain_text("Hello World");
                    }
                }
                #[salvo::async_trait]
                impl salvo::Handler for Hello {
                    #[inline]
                    async fn handle(
                        &self,
                        req: &mut salvo::Request,
                        depot: &mut salvo::Depot,
                        res: &mut salvo::Response,
                        ctrl: &mut salvo::routing::FlowCtrl
                    ) {
                        Self::handle(req, depot, res)
                    }
                }
            }
            .to_string()
        );
    }
}
