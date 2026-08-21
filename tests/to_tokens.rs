//! What `derive_display` generates, across the shapes an impl block can take.
//!
//! Deliberately imports nothing but the attribute and what the `ToTokens` impls
//! themselves need. The generated code names every path absolutely, so a caller should
//! not have to bring `Display`, `Formatter` or `quote` into scope to use it, and the
//! absence of those imports here is what checks that.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use derive_display::derive_display;

pub struct Plain;

#[derive_display]
impl ToTokens for Plain {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(Plain).to_tokens(tokens);
    }
}

#[test]
fn a_plain_type_displays_as_its_tokens() {
    assert_eq!(format!("{}", Plain), "Plain");
}

#[test]
fn the_original_impl_survives() {
    // The attribute adds to the item rather than replacing it, so the trait it read is
    // still implemented.
    let mut tokens = TokenStream::new();
    Plain.to_tokens(&mut tokens);
    assert_eq!(tokens.to_string(), "Plain");
}

#[test]
fn display_agrees_with_to_tokens() {
    assert_eq!(format!("{}", Plain), Plain.to_token_stream().to_string());
}

pub struct Bounded<T> {
    pub inner: T,
}

#[derive_display]
impl<T> ToTokens for Bounded<T>
where
    T: Clone,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(Bounded).to_tokens(tokens);
    }
}

#[test]
fn a_where_clause_reaches_the_generated_impl() {
    // The clause used to be dropped. `parsed.generics` prints the parameters and not the
    // bounds, so the generated `impl<T> Display for Bounded<T>` had no `T: Clone` while
    // its body called something that needed one, and it could not compile.
    let b = Bounded { inner: 1u8 };
    assert_eq!(format!("{b}"), "Bounded");
}

pub struct Inline<T: Clone> {
    pub inner: T,
}

#[derive_display]
impl<T: Clone> ToTokens for Inline<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(Inline).to_tokens(tokens);
    }
}

#[test]
fn an_inline_bound_reaches_the_generated_impl() {
    let i = Inline { inner: 1u8 };
    assert_eq!(format!("{i}"), "Inline");
}

pub struct Borrowing<'a> {
    pub inner: &'a str,
}

#[derive_display]
impl<'a> ToTokens for Borrowing<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(Borrowing).to_tokens(tokens);
    }
}

#[test]
fn a_lifetime_parameter_reaches_the_generated_impl() {
    let b = Borrowing { inner: "x" };
    assert_eq!(format!("{b}"), "Borrowing");
}

pub struct Several<'a, T, const N: usize>
where
    T: Clone,
{
    pub inner: &'a [T; N],
}

#[derive_display]
impl<'a, T, const N: usize> ToTokens for Several<'a, T, N>
where
    T: Clone,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(Several).to_tokens(tokens);
    }
}

#[test]
fn a_lifetime_a_type_and_a_const_parameter_together() {
    let values = [1u8, 2, 3];
    let s = Several { inner: &values };
    assert_eq!(format!("{s}"), "Several");
}

pub enum Choice {
    This,
    That,
}

#[derive_display]
impl ToTokens for Choice {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::This => quote!(This).to_tokens(tokens),
            Self::That => quote!(That).to_tokens(tokens),
        }
    }
}

#[test]
fn an_enum_displays_per_variant() {
    assert_eq!(format!("{}", Choice::This), "This");
    assert_eq!(format!("{}", Choice::That), "That");
}

#[test]
fn the_format_flags_reach_through() {
    // `Display` on the token stream is what does the work, so width and alignment behave
    // the way they would on any other displayed value.
    assert_eq!(format!("{:>10}", Plain), "     Plain");
    assert_eq!(format!("{:-^11}", Plain), "---Plain---");
}

#[test]
fn a_qualified_trait_path_is_read_by_its_last_segment_and_emitted_whole() {
    // Two things at once. The attribute reads the last segment to decide which trait this
    // is, and emits the path as the caller wrote it.
    //
    // The second half is what stops the generated code hard-naming `::quote::ToTokens`. A
    // crate that depends on quote under another name has no `quote` to resolve, and got
    // `could not find 'quote' in the list of imported crates` spanned at the attribute.
    // Whatever path the caller's own impl resolves through, the generated one resolves
    // through too, because it is the same path.
    assert_eq!(format!("{}", Qualified), "Qualified");
}

pub struct Qualified;

#[derive_display]
impl quote::ToTokens for Qualified {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(Qualified).to_tokens(tokens);
    }
}
