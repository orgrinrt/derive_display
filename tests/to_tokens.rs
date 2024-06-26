use std::fmt::Display;
use std::fmt::Formatter;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use derive_display::derive_display;

pub struct MyStruct;

#[derive_display]
impl ToTokens for MyStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token = quote!(MyStruct);
        token.to_tokens(tokens);
    }
}

#[test]
fn test_to_tokens_derive() {
    let m = MyStruct {};
    println!("{}", m);
    let f = format!("{}", m);
    assert_eq!("MyStruct", f);
}
