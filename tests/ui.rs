//! What the attribute refuses, and how it says so.
//!
//! Every one of these used to be a `panic!` inside the macro, which reaches the user as a
//! proc-macro panic naming no span and no line. They are `compile_error!` now, pointed at
//! the thing that is wrong, and this is what keeps them that way.

use std::fs;

/// How many cases this suite expects to find.
///
/// `trybuild` is handed a glob, and a glob matching nothing is not an error.
const EXPECTED_CASES: usize = 3;

#[test]
fn every_refusal_says_what_is_wrong() {
    let found = fs::read_dir("tests/ui")
        .expect("the compile-fail case directory")
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "rs"))
        .count();
    assert_eq!(found, EXPECTED_CASES, "expected {EXPECTED_CASES} cases, found {found}");

    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
