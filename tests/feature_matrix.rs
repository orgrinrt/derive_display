//! What each feature does to a consumer.
//!
//! This crate is a proc macro, so nothing about the features can be seen from inside it:
//! `no_std` changes which crate the generated code reaches `ToString` through, and the
//! generated code lands somewhere else. The only honest check is to compile a consumer.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Builds this crate under one feature selection.
fn check(features: &str) -> (bool, String) {
    let mut command = Command::new(env!("CARGO"));
    command
        .args(["check", "--quiet", "--no-default-features"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("CARGO_TARGET_DIR", concat!(env!("CARGO_MANIFEST_DIR"), "/target/feature-matrix"));
    if !features.is_empty() {
        command.args(["--features", features]);
    }
    let output = command.output().expect("cargo runs");
    (output.status.success(), String::from_utf8_lossy(&output.stderr).to_string())
}

/// Builds a throwaway crate whose body is `body`, against this crate at `features`.
fn consumer_compiles(name: &str, features: &str, attrs: &str, body: &str) -> (bool, String) {
    let root = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/target/consumers")).join(name);
    fs::create_dir_all(root.join("src")).expect("the consumer directory");

    let features_list = if features.is_empty() {
        String::new()
    } else {
        features.split(',').map(|f| format!("\"{f}\"")).collect::<Vec<_>>().join(", ")
    };

    fs::write(
        root.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{name}"
version = "0.0.0"
edition = "2021"

[dependencies.derive_display]
path = "{crate_dir}"
default-features = false
features = [{features_list}]

[dependencies]
quote = "1.0"
proc-macro2 = "1.0"

[workspace]
"#,
            crate_dir = env!("CARGO_MANIFEST_DIR"),
        ),
    )
    .expect("the consumer manifest");

    fs::write(root.join("src").join("lib.rs"), format!("{attrs}\n{body}\n"))
        .expect("the consumer source");

    let output = Command::new(env!("CARGO"))
        .args(["check", "--quiet"])
        .current_dir(&root)
        .env("CARGO_TARGET_DIR", concat!(env!("CARGO_MANIFEST_DIR"), "/target/consumers/target"))
        .output()
        .expect("cargo runs");

    (output.status.success(), String::from_utf8_lossy(&output.stderr).to_string())
}

/// A type with a `ToTokens` impl for the attribute to work from.
const SUBJECT: &str = r#"
use derive_display::derive_display;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct Marker;

#[derive_display]
impl ToTokens for Marker {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote!(marker));
    }
}
"#;

#[test]
fn every_selection_builds() {
    for selection in ["", "no_std", "no_alloc", "no_alloc,no_std"] {
        let (ok, err) = check(selection);
        let label = if selection.is_empty() { "no features" } else { selection };
        assert!(ok, "{label} builds:\n{err}");
    }
}

#[test]
fn a_std_consumer_gets_a_working_display() {
    let (ok, err) = consumer_compiles("std_consumer", "", "", SUBJECT);
    assert!(ok, "a plain consumer can use the attribute:\n{err}");
}

#[test]
fn a_no_std_consumer_gets_a_working_display() {
    // The reason the feature exists. The generated code named `::std::string::ToString`
    // unconditionally, so this did not compile at all, and no test inside this crate could
    // see it, because the path only has to resolve where the expansion lands.
    let (ok, err) = consumer_compiles(
        "no_std_consumer",
        "no_std",
        "#![no_std]\nextern crate alloc;",
        SUBJECT,
    );
    assert!(ok, "a `#![no_std]` consumer can use the attribute:\n{err}");
}

#[test]
fn the_std_path_is_what_breaks_a_no_std_consumer() {
    // The control for the test above, and the shape of the defect it fixes: the same
    // consumer, against the crate built for `std`, reaches a path that does not resolve
    // there.
    let (ok, err) = consumer_compiles(
        "no_std_against_std",
        "",
        "#![no_std]\nextern crate alloc;",
        SUBJECT,
    );
    assert!(!ok, "the std path must not resolve in a `#![no_std]` consumer");
    assert!(
        err.contains("std"),
        "the error is about `std` being unavailable:\n{err}",
    );
}

#[test]
fn the_no_std_path_needs_the_consumer_to_have_declared_alloc() {
    // The other half of why this is a feature rather than a default: `::alloc` resolves
    // only where the consumer declared it, and a plain `std` crate has no reason to have.
    // A consumer that picks `no_std` and forgets `extern crate alloc` is told so by name.
    let (ok, err) = consumer_compiles("no_std_without_alloc", "no_std", "#![no_std]", SUBJECT);
    assert!(!ok, "`::alloc` must not resolve without `extern crate alloc`");
    assert!(err.contains("alloc"), "the error names `alloc`:\n{err}");
}
