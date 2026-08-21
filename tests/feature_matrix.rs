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
        .env(
            "CARGO_TARGET_DIR",
            concat!(env!("CARGO_MANIFEST_DIR"), "/target/feature-matrix"),
        );
    if !features.is_empty() {
        command.args(["--features", features]);
    }
    let output = command.output().expect("cargo runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// Builds a throwaway crate whose body is `body`, against this crate at `features`.
fn consumer_compiles(name: &str, features: &str, attrs: &str, body: &str) -> (bool, String) {
    let root = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/target/consumers")).join(name);
    fs::create_dir_all(root.join("src")).expect("the consumer directory");

    let features_list = if features.is_empty() {
        String::new()
    } else {
        features
            .split(',')
            .map(|f| format!("\"{f}\""))
            .collect::<Vec<_>>()
            .join(", ")
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

    fs::write(
        root.join("src").join("lib.rs"),
        format!("{attrs}\n{body}\n"),
    )
    .expect("the consumer source");

    let output = Command::new(env!("CARGO"))
        .args(["check", "--quiet"])
        .current_dir(&root)
        .env(
            "CARGO_TARGET_DIR",
            concat!(env!("CARGO_MANIFEST_DIR"), "/target/consumers/target"),
        )
        .output()
        .expect("cargo runs");

    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
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
fn a_no_std_consumer_needs_no_extern_crate_alloc_of_its_own() {
    // The property the const-block expansion buys, and the reason this crate has no feature
    // selecting a path. The expansion declares `alloc` itself, so a consumer that says
    // `#![no_std]` and nothing else still gets a working `Display`.
    //
    // Before, `::alloc` resolved only where the consumer had declared it, and forgetting was
    // a compile error in generated code they did not write.
    let (ok, err) = consumer_compiles("no_std_bare", "", "#![no_std]", SUBJECT);
    assert!(
        ok,
        "a bare `#![no_std]` consumer gets a working Display:\n{err}"
    );
}

#[test]
fn one_build_of_this_crate_serves_both_kinds_of_consumer() {
    // The control on the pair above, and the whole point. Both consumers compile against the
    // *same* feature selection of this crate, which is what makes the emitted path additive:
    // cargo unifies features across a dependency graph, so a path that followed a feature
    // meant one sibling turning it on changed what every unrelated consumer's macros emit.
    //
    // Without this, the two tests above pass while each needs its own build, which is the
    // defect wearing the fix's clothes.
    let (std_ok, std_err) = consumer_compiles("both_std", "", "", SUBJECT);
    let (no_std_ok, no_std_err) = consumer_compiles("both_no_std", "", "#![no_std]", SUBJECT);
    assert!(std_ok, "the std consumer:\n{std_err}");
    assert!(
        no_std_ok,
        "the no_std consumer, same features:\n{no_std_err}"
    );
}

#[test]
fn the_expansion_names_neither_std_nor_a_bare_alloc() {
    // Read the source rather than a build, because this is a claim about what is emitted and
    // a passing build says only that whatever was emitted resolved somewhere. Both of the old
    // spellings are the defect: `::std` is unreachable from `#![no_std]`, and `::alloc`
    // unqualified is unreachable from a consumer that never declared it.
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/derive_display.rs"))
        .expect("the crate source");
    // Comment lines go first, because a comment can contain any string the code can and this
    // test anchors on strings. The first version split on `const _: () = {` and landed in the
    // comment that explains the const block, so it read the prose and reported the code
    // missing.
    let emitted: String = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !emitted.contains("::std::string::ToString"),
        "the expansion names `::std`, which no `#![no_std]` consumer can reach",
    );
    assert!(
        emitted.contains("extern crate alloc as __derive_display_alloc"),
        "the expansion declares alloc itself, under a name a consumer will not have taken",
    );
}
