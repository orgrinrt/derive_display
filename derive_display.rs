// The body of the `derive_display` attribute.
//
// Kept beside the manifest rather than under `src/`, and pulled in by
// `include_proc_macro`'s crate-relative path form, because this crate is also that
// crate's smallest worked example.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::ItemImpl;

/// Emits the original item, followed by a `Display` implementation derived from it.
pub fn derive_display(attr: TokenStream, item: TokenStream) -> TokenStream {
    let original: TokenStream2 = item.clone().into();

    // The attribute takes no arguments. Accepting and ignoring them means a typo in one
    // does nothing at all, which is the worst of the three available behaviours.
    if !attr.is_empty() {
        let attr: TokenStream2 = attr.into();
        return with_original(
            original,
            refuse(
                attr.span(),
                "`derive_display` takes no arguments. It reads the trait from the impl block \
                 it is attached to.",
            ),
        );
    }

    let parsed: ItemImpl = match syn::parse(item) {
        Ok(parsed) => parsed,
        // A parse error carries its own span and message, both better than anything
        // invented here.
        Err(e) => return e.to_compile_error().into(),
    };

    // Borrowed rather than cloned: syn is built without clone-impls here, deliberately,
    // and nothing in this needs to own the path.
    let Some((_bang, trait_path, _for)) = parsed.trait_.as_ref() else {
        return with_original(
            original,
            refuse(
                parsed.self_ty.span(),
                "`derive_display` needs an impl of a trait, and this block implements none. \
                 Attach it to something like `impl ToTokens for MyType`.",
            ),
        );
    };

    let Some(trait_name) = trait_path.segments.last().map(|s| s.ident.to_string()) else {
        return with_original(
            original,
            refuse(trait_path.span(), "the implemented trait has no name to read."),
        );
    };

    let derived = match trait_name.as_str() {
        "ToTokens" => from_to_tokens(&parsed),
        other => refuse(
            trait_path.span(),
            &format!(
                "`derive_display` cannot derive `Display` from `{other}`. It currently knows \
                 how to do so from `ToTokens` only."
            ),
        ),
    };

    with_original(original, derived)
}

/// The original item, then whatever was derived from it.
///
/// The original is emitted even when the derivation fails, so what a reader sees is one
/// error about the attribute rather than a cascade about a type that suddenly does not
/// exist.
fn with_original(original: TokenStream2, derived: TokenStream2) -> TokenStream {
    quote! { #original #derived }.into()
}

fn refuse(span: Span, message: &str) -> TokenStream2 {
    quote_spanned! { span => compile_error!(#message); }
}

/// `Display` written in terms of `ToTokens`.
///
/// Every path is absolute. The generated code lands in the caller's module, where nothing
/// promises that `Display`, `Formatter` or `quote` is in scope, and demanding three
/// imports for an attribute that could write them itself is a poor trade.
///
/// It also no longer expands a `quote!` in the caller's crate. `ToTokens` already provides
/// `to_token_stream`, so the body is a conversion rather than a macro invocation around an
/// interpolated `self`.
fn from_to_tokens(parsed: &ItemImpl) -> TokenStream2 {
    let ty = &parsed.self_ty;

    // `split_for_impl` is what carries the `where` clause. Emitting `parsed.generics`
    // whole prints the parameters and drops the bounds, so
    // `impl<T> ToTokens for W<T> where T: Clone` produced a `Display` impl with no bound,
    // which could not compile. The type generics are not needed: the self type already
    // carries them.
    let (impl_generics, _ty_generics, where_clause) = parsed.generics.split_for_impl();

    quote! {
        impl #impl_generics ::core::fmt::Display for #ty #where_clause {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                // `f.pad` rather than writing straight out, so width, fill, alignment
                // and precision behave the way a reader expects of any Display. A
                // TokenStream's own Display ignores them, so delegating to it dropped
                // `{:>10}` on the floor.
                f.pad(&::std::string::ToString::to_string(
                    &::quote::ToTokens::to_token_stream(self),
                ))
            }
        }
    }
}
