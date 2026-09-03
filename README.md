# `derive_display`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/derive_display.svg)](https://github.com/orgrinrt/derive_display/stargazers)
[![Crates.io](https://img.shields.io/crates/v/derive_display)](https://crates.io/crates/derive_display)
[![docs.rs](https://img.shields.io/docsrs/derive_display)](https://docs.rs/derive_display)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/derive_display.svg)](https://github.com/orgrinrt/derive_display/issues)
![License](https://img.shields.io/github/license/orgrinrt/derive_display?color=%23009689)

> An attribute that derives `Display` from the `ToTokens` impl it sits on.

</div>

A type that already knows how to write itself out as tokens, which is most of what goes on
inside a code generator or a proc macro, has its `Display` sitting there in the same impl
already, and this attribute is what says so. Put `#[derive_display]` on an
`impl ToTokens for T` block and a `Display` for `T` comes out beside it, formatting whatever
`to_token_stream` produces, with the original impl left as it was.

The generated impl goes through `Formatter::pad`, so width, fill, alignment and precision
behave the way they do on any other value, which is the part a hand-written forwarding tends
to miss, since a `TokenStream`'s own `Display` takes none of those into account. The generics
come along too, the `where` clause included, and every path in what gets emitted is absolute,
so the site imports the attribute and whatever the `ToTokens` impl itself needed, and
nothing else on top.

Do note that this is not a general `Display` derive. `derive_more` has one of those, built off
a format string over the fields, and it's the better fit where the type has no `ToTokens` to
begin with. What this one is for is the case where the tokens are the display, and writing the
same thing twice is the busywork being removed.

## Usage

```bash
cargo add derive_display
```

`cargo add` writes `derive_display = "0.0.5"`, and on a `0.0.x` version that resolves to that
one and nothing else, so what it wrote is the pin. Getting the next one means changing the
number by hand.

The attribute goes on the impl block, takes no arguments, and reads the trait from the block
itself:

```rust
use derive_display::derive_display;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

struct Marker;

#[derive_display]
impl ToTokens for Marker {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(marker).to_tokens(tokens);
    }
}

assert_eq!(format!("{Marker}"), "marker");
assert_eq!(format!("[{Marker:>8}]"), "[  marker]");
```

`ToTokens` is the only trait it knows how to derive from right now, so anything else on the
block is refused. The refusals are compile errors pointed at the thing that is wrong, and
there are three of them, an argument given to the attribute, an impl block with no trait on
it, and a trait other than `ToTokens`. In each case the original impl is still emitted, so
what comes back is one error about the attribute rather than a cascade about a type that
suddenly has no `to_tokens`.

The trait is recognised by the last segment of its path and emitted spelled the way it was
written, so `impl quote::ToTokens for T` works and resolves through the same path the impl
does, though it also means a trait named `ToTokens` that isn't quote's gets accepted, and the
generated body then calls `to_token_stream` on it and fails there instead.

## Example

Here's the kind of thing this turns up in. A small piece of a generator holds the fields it is
about to write into a struct, each one a `ToTokens` so the `quote!` further up can interpolate
it, and at some point the same fields want printing in a diagnostic, a log line or a test
assertion, padded into a column. The `ToTokens` is already the right text, so the `Display` is
derived off it:

```rust
use derive_display::derive_display;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

// what one generated field looks like, before it's a field
struct Field {
    name: &'static str,
    ty: &'static str,
}

#[derive_display]
impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = Ident::new(self.name, Span::call_site());
        let ty = Ident::new(self.ty, Span::call_site());
        tokens.extend(quote!(pub #name: #ty));
    }
}

// a bound on the parameter is carried onto the generated Display as well
struct Listed<T> {
    items: Vec<T>,
}

#[derive_display]
impl<T> ToTokens for Listed<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let items = &self.items;
        tokens.extend(quote!(#(#items),*));
    }
}

let retries = Field { name: "retries", ty: "u32" };
let timeout = Field { name: "timeout_ms", ty: "u64" };

// the Display, which nothing above wrote
assert_eq!(retries.to_string(), "pub retries : u32");

// and it is a real Display, so the specifiers behave
assert_eq!(format!("[{retries:>20}]"), "[   pub retries : u32]");
assert_eq!(format!("[{retries:-<20}]"), "[pub retries : u32---]");

// the same value through a TokenStream's own Display ignores the width
let raw = retries.to_token_stream();
assert_eq!(format!("[{raw:>20}]"), "[pub retries : u32]");

let fields = Listed { items: vec![retries, timeout] };
assert_eq!(fields.to_string(), "pub retries : u32 , pub timeout_ms : u64");
```

What the attribute writes for `Field`, beside the impl it was given, is this:

```rust,ignore
const _: () = {
    extern crate alloc as __derive_display_alloc;

    impl ::core::fmt::Display for Field {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.pad(&__derive_display_alloc::string::ToString::to_string(
                &ToTokens::to_token_stream(self),
            ))
        }
    }
};
```

The `extern crate alloc` inside the `const` block is what makes one expansion resolve in a
`#![no_std]` crate and a plain one alike, since `ToString` lives in `alloc` and a `std` crate
has no reason to have declared that itself. A `#![no_std]` consumer needs no `extern crate
alloc` of its own for this to work.

## Motivation

`Display` isn't implemented for anything by itself, which is right, since most types have no
one obvious text form, but a type whose whole job is to become tokens does have one, and it's
the tokens. Writing the forwarding impl by hand is a handful of lines, and it's the same
handful every time, and the obvious spelling of it, handing the formatter off to the
`TokenStream`'s `Display`, quietly drops every width and alignment flag a caller passes. So
the impl either gets copied around with that defect in it, or gets written properly once and
then copied around anyway, with the two drifting apart at some point most likely.

The attribute is that impl written once, with the padding done right, the generics and bounds
carried through from the block it reads, and the paths qualified so it lands anywhere. What it
costs is one `String` per format call, because `pad` needs the whole text before it can pad
it, and a proc macro in the build, though the `quote` that brings is there already in any
crate that implements `ToTokens`, and `syn` most likely too.

The crate itself is small enough to double as the worked example for
[`include_proc_macro`](https://crates.io/crates/include_proc_macro), which is what wires the
attribute body into the crate root here.

## Extras

### Status

Pre-1.0, so the api hasn't settled and a release can still move things, though the surface is
one attribute and there isn't much room for it to move in. Every release is tagged and the log
between two tags is what changed. It builds on stable, `rust-version` is 1.71, and no nightly
feature is involved anywhere.

### Cargo features

There are two, `no_std` and `no_alloc`, both off by default, and both inert. The generated
impl declares `alloc` itself, so the same expansion already works in a `#![no_std]` consumer
without either being turned on, and there's nothing left for a feature to select. They exist
so a consumer can state what it is and have that stated in the manifest, and `no_alloc` implies
`no_std`. Do note that what the attribute generates still allocates, whichever is on, since
`to_token_stream` builds a `TokenStream` and `pad` wants the text whole. That is a property of
`ToTokens` rather than of this crate.

Neither touches the crate itself, which is a proc macro and always `std`, as it runs on the
host inside the compiler, where `syn`, `quote` and `proc-macro2` are all `std` as well.

### Limitations

`ToTokens` is the only source it derives from. The `Display` costs an allocation per call, as
above. And the attribute has to sit on an impl of a trait, so a plain inherent `impl T` block
is refused.

## Support

Feel free to contribute! If unsure about wasting work, the best practice is to throw in an issue describing what you'd do, and only then commit to writing a big PR, because chances are, it might not be something that belongs here. However, forks are always a valid choice and we'd encourage everyone to experiment and have their own takes on this. When doing this, do mind the license(s) though!

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/derive_display/blob/main/LICENSE)
