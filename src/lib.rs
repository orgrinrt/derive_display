#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "README.md"))]

include_proc_macro::attr_macro!(derive_display -> @"derive_display.rs"::derive_display);
