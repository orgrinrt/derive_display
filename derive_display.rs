use proc_macro::TokenStream;

use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::ItemImpl;
use syn::{parse_str, Generics, Path, Type};

#[proc_macro_attribute]
pub fn derive_display(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // panic!("item: \n {} \n", item);
    let mut result = TokenStream2::new();
    let orig_item = item.clone();
    TokenStream2::from(item).to_tokens(&mut result);

    let parsed: ItemImpl = syn::parse(orig_item).expect("Expected a valid ItemImpl block");
    // let (_, tr, _) = parsed.trait_.expect("Expected there to be a trait in an ItemImpl");
    let impl_generics: Generics = parsed.generics;
    let ty = *parsed.self_ty;
    let tr_path: Path = parsed
        .trait_
        .expect("Expected a valid trait implementation")
        .1;
    let tr: Ident = tr_path
        .segments
        .last()
        .expect("Expected a valid path for the implemented trait")
        .ident
        .clone();
    let q = match tr.to_string().as_str() {
        "ToTokens" => derive_from_to_tokens(impl_generics, ty),
        _ => {
            panic!("This implementation is not yet supported by `derive_display`")
        },
    };

    q.to_tokens(&mut result);
    result.into()
}

fn derive_from_to_tokens(impl_generics: Generics, ty: Type) -> TokenStream2 {
    let inner_s: TokenStream2 = parse_str("#s").expect("Expected a valid TokenStream");
    let q = quote! {
        impl #impl_generics Display for #ty {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let s = &self;
                let q = quote!(#inner_s);
                f.write_fmt(format_args!("{}", q))
            }
        }
    };
    q
}
