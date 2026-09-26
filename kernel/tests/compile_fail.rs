//! Guarantees checked by compiling code that must not compile.
//!
//! `INV-7` is the reason this file exists. The claim is not that a cross-system write is caught
//! at run time — it is that a system cannot *name* one, so the program that violates the
//! single-writer rule does not exist. A claim of that shape cannot be asserted from a test that
//! runs; the only way to test it is to compile the violation and require the compiler to refuse.
//!
//! Each case is an attempt an ordinary pack author could make without `unsafe` and without
//! editing the crate that owns the state, and each `.stderr` file pins the *reason* the compiler
//! refused rather than merely that it did:
//!
//! - a system cannot write a component another system owns;
//! - a write token cannot be constructed, or built out of its fields — only granted;
//! - a pack cannot turn another crate's type into a component, which is what confines a
//!   dishonest ownership claim to the claiming pack's own state;
//! - taking another system's declared name does not take its ownership;
//! - a system type that keeps its constructor to itself cannot be impersonated.
//!
//! The attempts that are *not* here, because they compile, are recorded as residual boundaries in
//! §2.10.2 of `.structured-coding/plans/mvp0/step-03-kernel-and-systems.md`.

#[test]
fn the_single_writer_rule_is_enforced_by_the_compiler() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs");
}
