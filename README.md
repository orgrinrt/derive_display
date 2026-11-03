# `derive_display`

[![Crates.io](https://img.shields.io/crates/v/derive_display)](https://crates.io/crates/derive_display) [![docs.rs](https://img.shields.io/docsrs/derive_display)](https://docs.rs/derive_display)

> An attribute that derives `Display` from the `ToTokens` impl it sits on.

A type that already knows how to write itself out as tokens, which is most of what lives inside a code generator or a proc macro, has its `Display` in the same impl already. Put `#[derive_display]` on an `impl ToTokens for T` block and a `Display` for `T` comes out beside it, formatting whatever `to_token_stream` produces, with the original impl left as it was. It isn't a general `Display` derive, `derive_more` has one of those built on a format string over the fields, and this one is for the case where the tokens are the display.

The generated impl goes through `Formatter::pad`, so width, fill, alignment and precision work as on any other value, which a plain forward to the `TokenStream`'s own `Display` doesn't do. Generics and the `where` clause are carried over, every path in the output is absolute, and the expansion declares `alloc` itself so it works in a `#![no_std]` consumer as well.

## Usage

```bash
cargo add derive_display
```

The attribute goes on the impl block and takes no arguments.

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

`ToTokens` is the only trait it derives from, and an argument to the attribute, an impl with no trait and any other trait are compile errors pointing at the thing that is wrong, with the original impl still emitted so there is one error and no cascade. The trait is recognised by the last segment of its path, so `impl quote::ToTokens for T` works, and so would an unrelated trait that happens to be called `ToTokens`, which then fails in the generated body.

## Status

Pre-1.0 and the surface is one attribute, so there isn't much room for it to move. It builds on stable from rust 1.71. The `no_std` and `no_alloc` features exist so a consumer can state what it is in its manifest and both are inert, and what the attribute generates allocates either way, one `String` per format call, because `pad` needs the whole text first. The crate also doubles as the worked example for [`include_proc_macro`](https://crates.io/crates/include_proc_macro).

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/derive_display/blob/main/LICENSE)
