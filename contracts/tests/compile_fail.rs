//! Guarantees checked by compiling code that must not compile.
//!
//! Some of this crate's promises cannot be asserted at runtime, because a program that violates
//! them does not exist: distinct identities are not interchangeable, a typed entity reference
//! cannot be fabricated from a raw [`EntityId`](mineworld_contracts::EntityId), and an entity's
//! lifecycle cannot be assigned past its transition check. The cases under
//! `tests/compile_fail/` are the executable form of those promises, and each one's `.stderr`
//! file pins the *reason* the compiler rejects it — not merely that it did.

#[test]
fn guarantees_the_compiler_owns_are_actually_enforced() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs");
}
