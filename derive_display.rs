use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::{Generics, parse_str};
use syn::__private::TokenStream2;
use syn::ItemImpl;

#[proc_macro_attribute]
pub fn derive_display(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // panic!("item: \n {} \n", item);
    let mut result = TokenStream2::new();
    let orig_item = item.clone();
    TokenStream2::from(item).to_tokens(&mut  result);
    
    let parsed: ItemImpl = syn::parse(orig_item).expect("Expected a valid ItemImpl block");
    // let (_, tr, _) = parsed.trait_.expect("Expected there to be a trait in an ItemImpl");
    let impl_generics: Generics = parsed.generics;
    let ty = *parsed.self_ty;
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
    q.to_tokens(&mut  result);
    result.into()
}


// impl<T: Parse + Display + ToTokens + Clone, D: Token + Parse + Default + ToTokens> Display for
// PunctSet<T,
//     D> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let s = &self;
//         let q = quote!(#s);
//         f.write_fmt(format_args!("{}", q.to_string()))
//     }
// }
