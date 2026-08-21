//! The smallest thing the attribute does: one `ToTokens` impl, one `Display` for free.
//!
//! ```text
//! cargo run --example one_impl
//! ```

use derive_display::derive_display;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// A field in something being generated.
struct Field {
    name: &'static str,
    ty: &'static str,
}

// The attribute emits this impl unchanged, then a `Display` written in terms of it.
#[derive_display]
impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = syn::Ident::new(self.name, proc_macro2::Span::call_site());
        let ty = syn::Ident::new(self.ty, proc_macro2::Span::call_site());
        tokens.extend(quote!(pub #name: #ty));
    }
}

fn main() {
    let field = Field {
        name: "retries",
        ty: "u32",
    };

    // `Display`, which nothing in this file wrote.
    println!("{field}");

    // And it is a real `Display`, so the format specifiers work. That is the reason the
    // generated impl goes through `f.pad` rather than handing off to a `TokenStream`'s own
    // `Display`, which ignores every one of them.
    println!("[{field:>30}]");
    println!("[{field:<30}]");
    println!("[{field:^30}]");
    println!("[{field:*^30}]");

    // Which is what the same value looks like without the attribute: the width is dropped.
    let raw = field.to_token_stream();
    println!("[{raw:>30}]  (a TokenStream's own Display, for comparison)");
}
