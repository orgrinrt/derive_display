derive_display
============
<div style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/derive_display.svg)](https://github.com/orgrinrt/derive_display/stargazers)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/derive_display)](https://crates.io/crates/derive_display)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/derive_display.svg)](https://github.com/orgrinrt/derive_display/issues)
[![Current Version](https://img.shields.io/badge/version-0.0.3-red.svg)](https://github.com/orgrinrt/derive_display)

> An attribute to derive a `Display` implementation from another trait implementation, currently `ToTokens`.

</div>

## Usage

To use this proc-macro in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
derive_display = "0.0.3" # use the latest version, or a specific one if needed
```

Then wherever you want to derive `Display` from another implementation, use the `#[derive_display]`
attribute before that implementation:

```rust
use derive_display::derive_display;

#[derive_display]
impl ToTokens for MyStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // ...
    }
}
```

Tag the implementation with the attribute, and nothing else needs to be done. The original
implementation is kept as-is; a `Display` implementation is emitted alongside it.

### Currently supported implementations

`ToTokens` is the only supported implementation right now: the derived `Display` formats whatever
the `to_tokens` method produces. Tagging an implementation of any other trait fails at compile time.

> Note: Support for generics and some usual edge cases is built in.
> Issues and PRs are welcome if it doesn't cover something yet.

## Example

Let's say we have a struct and we need to provide a `Display` implementation. If it so happens
that we already have a suitable implementation that formats a string for some other purpose, and that would suffice, you
can tag the impl with the `#[derive_display]` attribute:

```rust
use std::fmt::{Display, Formatter};

use derive_display::derive_display;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

struct MyStruct {
    x: i32,
    y: i32,
}

#[derive_display]
impl ToTokens for MyStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let content = format!("MyStruct: x = {}, y = {}", self.x, self.y);
        content.to_tokens(tokens);
    }
}
```

This expands to the following implementation, *in addition* to the source implementation:

```rust
impl Display for MyStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = &self;
        let q = quote!(#s); // tokens from the `ToTokens` impl
        f.write_fmt(format_args!("{}", q))
    }
}
```

Note that the generated code refers to `Display`, `Formatter` and `quote!` unqualified, so those
need to be in scope where the attribute is used (as in the imports above).

This results in a `Display` implementation without any explicit busywork. Without this crate (or some other way of
achieving the same), you would have to manually write out something like the above implementation of
`Display`, which is redundant, since it just forwards another implementation.

## The Problem

In Rust, types do not automatically implement the `Display` trait, which is required for types to be printable. This can
feel like unnecessary busywork, if we just want to print something in a
human-readable form, *especially* if we already have implementations for traits that construct suitable formatted
strings representing our struct.

There are derive proc-macros for display in the ecosystem (e.g. `#[derive(Display)]` from `derive_more`), but they are
not always ideal, and sometimes the struct just has members incompatible with those macros.

This crate provides a solution to this by deriving the `Display`
implementation from an existing trait implementation with a single attribute.

This creates a `Display` implementation with minimal effort, and removes the need for a verbose and manual `Display`
implementation, especially when it would mirror an already implemented `ToTokens` trait.

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying
me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> You can check out the full license [here](https://github.com/orgrinrt/derive_display/blob/main/LICENSE)

This project is licensed under the terms of the **MIT** license.
