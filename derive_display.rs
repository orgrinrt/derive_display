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
        "ToTokens" => from_to_tokens(&parsed, trait_path),
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
/// Every path in the generated code is absolute, except the trait's, which is taken from
/// the impl block being decorated. The generated code lands in the caller's module, where
/// nothing promises that `Display` or `Formatter` is in scope, so the attribute writes the
/// full paths itself instead of asking for imports it could have spelled out.
///
/// The trait is spelled the way the caller spelled it and not as `::quote::ToTokens`, since
/// a hard-coded crate name assumes the consumer named their dependency the same way, and a
/// crate depending on `quote` under some other name has no `quote` to resolve. Whatever
/// path the caller's own impl resolves through, the generated one resolves through too.
///
/// The body is a plain call to `to_token_stream`, which `ToTokens` provides already, so
/// nothing gets expanded in the caller's crate beyond the impl itself.
fn from_to_tokens(parsed: &ItemImpl, trait_path: &syn::Path) -> TokenStream2 {
    let ty = &parsed.self_ty;

    // `split_for_impl` is what carries the `where` clause. Emitting `parsed.generics`
    // whole prints the parameters and drops the bounds, so
    // `impl<T> ToTokens for W<T> where T: Clone` would give a `Display` impl with no bound
    // on `T`, and a body that needs one. The type generics are not needed, since the self
    // type already carries them.
    let (impl_generics, _ty_generics, where_clause) = parsed.generics.split_for_impl();

    // `ToString` lives in `alloc` and is re-exported by `std`. Naming `::std` puts the impl
    // out of reach of a `#![no_std]` consumer, and naming `::alloc` puts it out of reach of a
    // plain `std` consumer, which has no reason to have declared `extern crate alloc`.
    //
    // So the expansion declares it itself, inside a `const _: () = { .. }`. An `extern crate`
    // is legal in a block, an impl written in one is as global as any other, and `alloc` is
    // present on a `std` target too. One spelling therefore serves both, which is why this
    // crate has no feature selecting between them: a feature here would have been read as
    // additive and is not, since cargo unifies features across a dependency graph and one
    // sibling turning it on would change what every unrelated consumer's macros emit.
    quote! {
        const _: () = {
            extern crate alloc as __derive_display_alloc;

            impl #impl_generics ::core::fmt::Display for #ty #where_clause {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    // `f.pad` and not a straight write, so width, fill, alignment and
                    // precision behave the way they do on any other Display. A
                    // TokenStream's own Display ignores all of them, so delegating to it
                    // would lose a `{:>10}` entirely.
                    f.pad(&__derive_display_alloc::string::ToString::to_string(
                        &#trait_path::to_token_stream(self),
                    ))
                }
            }
        };
    }
}

