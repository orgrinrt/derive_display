use derive_display::derive_display;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct Thing;

#[derive_display(from = "ToTokens")]
impl ToTokens for Thing {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(Thing).to_tokens(tokens);
    }
}

fn main() {}
