//! The examples are built by `cargo test` and never run by it, so they are run here.
//!
//! For this crate the risk is specific: the generated `Display` goes through `f.pad` so
//! that width, fill and alignment work, and the obvious implementation, handing off to a
//! `TokenStream`'s own `Display`, silently drops all three. That difference is invisible in
//! a test that only formats with `{}`.

use std::process::Command;

/// Runs one example and returns what it printed.
fn run_example(name: &str) -> String {
    let output = Command::new(env!("CARGO"))
        .args(["run", "-q", "--example", name])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env(
            "CARGO_TARGET_DIR",
            concat!(env!("CARGO_MANIFEST_DIR"), "/target/examples"),
        )
        .output()
        .unwrap_or_else(|e| panic!("could not run example {name}: {e}"));

    assert!(
        output.status.success(),
        "example {name} exited {}\n--- stderr\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
    );

    String::from_utf8(output.stdout).expect("example printed something that is not utf-8")
}

#[test]
fn one_impl_formats_and_respects_every_specifier() {
    let out = run_example("one_impl");

    assert!(
        out.contains("pub retries : u32"),
        "the plain form is missing:\n{out}"
    );

    // Each specifier, at width 30. A `Display` that ignored them would print the same
    // 17 characters four times, so the widths are what says `f.pad` is doing its job.
    assert!(
        out.contains("[             pub retries : u32]"),
        "right align:\n{out}"
    );
    assert!(
        out.contains("[pub retries : u32             ]"),
        "left align:\n{out}"
    );
    assert!(
        out.contains("[      pub retries : u32       ]"),
        "centre:\n{out}"
    );
    assert!(
        out.contains("[******pub retries : u32*******]"),
        "fill character:\n{out}"
    );

    // And the comparison line, which is the same value through a `TokenStream`'s own
    // `Display` and comes out unpadded. Without this the four above could pass against an
    // implementation that padded by accident.
    assert!(
        out.contains("[pub retries : u32]  (a TokenStream's own Display"),
        "the unpadded comparison is missing:\n{out}",
    );
}

#[test]
fn generics_and_refusals_carries_bounds_through() {
    let out = run_example("generics_and_refusals");

    // A bound in a `where` clause, a bound written inline, and a lifetime. Each of these
    // is an impl that would not compile if the generated `Display` had the wrong generics.
    assert!(out.contains("Wrapped (42)"), "the generic wrapper:\n{out}");
    assert!(out.contains("1 , 2"), "the two-parameter case:\n{out}");
    assert!(
        out.contains("\"borrowed text\""),
        "the lifetime case:\n{out}"
    );

    // Nesting, and padding over a nested value.
    assert!(
        out.contains("Wrapped (Wrapped (42))"),
        "the nested case:\n{out}"
    );
    assert!(
        out.contains("[                  Wrapped (Wrapped (42))]"),
        "padding over a nested value:\n{out}",
    );
}
