derive_display
============
<div style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/derive_display.svg)](https://github.com/orgrinrt/derive_display/stargazers)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/derive_display)](https://crates.io/crates/derive_display)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/derive_display.svg)](https://github.com/orgrinrt/derive_display/issues)
[![Current Version](https://img.shields.io/badge/version-0.0.5-red.svg)](https://github.com/orgrinrt/derive_display)

> An attribute to derive a `Display` implementation from another trait implementation, currently `ToTokens`.

</div>

## Usage

To use this proc-macro in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
derive_display = "0.0.5" # use the latest version, or a specific one if needed
```

Then wherever you want to derive `Display` from another implementation, use the `#[derive_display]`
attribute before that implementation:

```rust
# use proc_macro2::TokenStream;
# use quote::{quote, ToTokens};
# struct MyStruct;
use derive_display::derive_display;

#[derive_display]
impl ToTokens for MyStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        quote!(MyStruct).to_tokens(tokens);
    }
}
# fn main() { assert_eq!(format!("{}", MyStruct), "MyStruct"); }
```

Tag the implementation with the attribute, and nothing else needs to be done. The original
implementation is kept as-is; a `Display` implementation is emitted alongside it.

### Currently supported implementations

`ToTokens` is the only supported implementation right now: the derived `Display` formats
whatever the `to_tokens` method produces. Tagging any other trait, an impl block with no
trait at all, or passing arguments to the attribute, each fails at compile time with a
message pointing at the thing that is wrong.

Generic impls are supported, including `where` clauses, inline bounds, lifetimes and const
parameters. Each of those shapes has a test, so a bounded impl generates a `Display` that
carries the same bounds.

## Example

Let's say we have a struct and we need to provide a `Display` implementation. If it so happens
that we already have a suitable implementation that formats a string for some other purpose, and that would suffice, you
can tag the impl with the `#[derive_display]` attribute:

```rust
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

```rust,ignore
impl ::core::fmt::Display for MyStruct {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.pad(&::std::string::ToString::to_string(
            &::quote::ToTokens::to_token_stream(self),
        ))
    }
}
```

Every path in it is absolute, so nothing has to be in scope where the attribute is used.
Earlier versions named `Display`, `Formatter` and `quote!` unqualified and needed three
imports at the call site to work.

`f.pad` is what makes the format flags behave: `{:>10}` and `{:-^11}` pad and align the way
they would on any other value. Writing the tokens straight out would drop them, which both
the previous implementation and a plain delegation to `TokenStream`'s own `Display` do.

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

## Features

Neither changes this crate, which is a proc macro and stays `std` whatever is selected: it
runs on the host inside the compiler, where syn, quote and proc-macro2 all use `std`. What
they change is the code the attribute writes into your crate.

| Feature | Effect on the generated `Display` |
|---|---|
| default | Reaches `ToString::to_string` through `::std::string`. |
| `no_std` | Says the consumer is `#![no_std]`. Changes nothing about what is emitted: the generated impl declares `alloc` itself, so one expansion resolves in a `#![no_std]` crate and a plain one alike. |
| `no_alloc` | Implies `no_std`, and governs this crate's own surface, which is empty. |

The generated `Display` still allocates and cannot stop: `ToTokens::to_token_stream`
materialises a `TokenStream`, and `Display` has to have the text before it can pad it to a
width. That is a property of the trait being derived from rather than of this crate, and
`no_alloc` says so here rather than implying otherwise by existing quietly.

`tests/feature_matrix.rs` compiles a real consumer under each selection, including a bare
`#![no_std]` one that declares no `alloc` of its own. Its control is the pair that matters:
both kinds of consumer compile against the *same* build of this crate, which is what makes the
emitted path additive rather than something a sibling can change under you.

## Examples

```text
cargo run --example one_impl
cargo run --example generics_and_refusals
```

The first is one `ToTokens` impl and the `Display` it gets for free, then that `Display`
under every format specifier, with a `TokenStream`'s own `Display` beside it for comparison
so the difference is visible. The second carries bounds through: a `where` clause, an
inline bound, a lifetime, and nesting. Both are run by `cargo test`, in
`tests/examples_run.rs`.

## License

> You can check out the full license [here](https://github.com/orgrinrt/derive_display/blob/main/LICENSE)

This project is licensed under the terms of the **MPL-2.0** license.
