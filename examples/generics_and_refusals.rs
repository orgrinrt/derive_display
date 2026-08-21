//! The attribute over generics and where clauses, and what it does when it cannot help.
//!
//! ```text
//! cargo run --example generics_and_refusals
//! ```

use derive_display::derive_display;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// A wrapper whose `ToTokens` needs a bound.
struct Wrapped<T> {
    inner: T,
}

// The `where` clause is carried onto the generated `Display`. Dropping it produced an impl
// that could not compile, because the body needs exactly the bound the original had.
#[derive_display]
impl<T> ToTokens for Wrapped<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = &self.inner;
        tokens.extend(quote!(Wrapped(#inner)));
    }
}

/// Two parameters and a bound on one of them.
struct Pair<A, B> {
    left: A,
    right: B,
}

#[derive_display]
impl<A: ToTokens, B> ToTokens for Pair<A, B>
where
    B: ToTokens + Clone,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (left, right) = (&self.left, &self.right);
        tokens.extend(quote!(#left , #right));
    }
}

/// A lifetime, which the self type carries and the generated impl must not repeat.
struct Borrowed<'a> {
    text: &'a str,
}

#[derive_display]
impl<'a> ToTokens for Borrowed<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let text = self.text;
        tokens.extend(quote!(#text));
    }
}

fn main() {
    println!("A generic wrapper, its bound carried onto the Display impl.\n");

    // `proc_macro2::Literal` rather than one of syn's literal types, because syn is
    // built here without `clone-impls` and the `Pair` below asks for `Clone`.
    let lit = proc_macro2::Literal::u32_unsuffixed(42);
    let wrapped = Wrapped {
        inner: lit.clone(),
    };
    println!("{wrapped}");

    println!("\nTwo parameters, one bounded inline and one in the where clause.\n");

    let pair = Pair {
        left: proc_macro2::Literal::u32_unsuffixed(1),
        right: proc_macro2::Literal::u32_unsuffixed(2),
    };
    println!("{pair}");

    println!("\nA lifetime, which the self type already carries.\n");

    let owned = String::from("borrowed text");
    let borrowed = Borrowed {
        text: &owned,
    };
    println!("{borrowed}");

    println!("\nNesting, since a Wrapped<T> is itself a ToTokens and so is its Display.\n");

    let nested = Wrapped {
        inner: Wrapped {
            inner: lit,
        },
    };
    println!("{nested}");
    println!("[{nested:>40}]");

    println!(
        "\nWhat it refuses is a compile error rather than a runtime one. \
         `tests/ui/` holds those cases:\n\
         \x20 an attribute given arguments, an impl of no trait, and a trait it cannot \
         derive from."
    );
}
