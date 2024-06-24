derive_display
============
<div style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/derive_display.svg)](https://github.com/orgrinrt/derive_display/stargazers)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/derive_display)](https://crates.io/crates/derive_display)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/derive_display.svg)](https://github.com/orgrinrt/derive_display/issues)
[![Current Version](https://img.shields.io/badge/version-0.0.2-red.svg)](https://github.com/orgrinrt/derive_display)

> A convenient attribute to derive `Display` implementation from another implementation. Currently supports `ToTokens`.

</div>

## Usage

To use this proc-macro in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
derive_display = "0.0.1" # use the latest version, or a specific one if needed
```

Then wherever you want to derive `Display` from another implementation, use the `#[derive_display]`
attribute before that implementation:

```rust
use derive_display::derive_display;

#[derive_display]
impl SomeTrait for MyStruct {
    fn some_trait(&self, foo: &mut Bar) {
        ...
    }
}
```

This means you can simply tag the implementation you wish with the attribute, and nothing else needs
to be done. Simple as that.

### Currently supported implementations

| Implementation | Description                                                           |
|----------------|-----------------------------------------------------------------------|
| `ToTokens`     | Simply uses whatever `ToTokens` returns as the display representation |

> Note: Support for generics and most usual edge cases are built in, so it *should* be as easy as plug in and use.
> Issues and PRs are welcome if it doesn't cover something yet!

## Example

Let's say we have a struct and we need to provide a `Display` implementation. If it so happens
that we already have a suitable implementation that formats a string for some other purpose, and that would suffice, you
can simply tag the impl with the `#[derive_display]` attribute:

```rust
use derive_display::derive_display;
use quote::ToTokens;

struct MyStruct {
    x: i32,
    y: i32,
}

#[derive_display]
impl ToTokens for MyStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
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
        let q = quote!(#s); // <-- this is for the `ToTokens` derive
        f.write_fmt(format_args!("{}", q.to_string()))
    }
}
```

This results in a `Display` implementation without any explicit busywork. Without this crate (or some other way of
achieving the same), the developer would have to manually write out something like the above implementation of
`Display`, which is pretty boring and redundant, since it just uses some method of forwarding another implementation.

## The Problem

In Rust, types do not automatically implement the `Display` trait, which is required for types to be printable. This
can feel like unnecessary busywork, if we simply want to print something sufficiently representing in a more
human-readable form, *especially* if we already have implementations for traits that construct suitable formatted
strings representing our struct.

This crate provides a solution to this by allowing developers to derive the `Display`
implementation from an existing trait implementation simply by using a simple attribute.

This creates a `Display` implementation with minimal effort, and removes the need for verbose and manual `Display`
implementation, especially when it's similar to already implemented `ToTokens` trait.

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying
me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> You can check out the full license [here](https://github.com/orgrinrt/derive_display/blob/master/LICENSE)

This project is licensed under the terms of the **MIT** license.
