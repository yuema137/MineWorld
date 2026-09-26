//! Type-level guarantees, checked by compiling code that must not compile.
//!
//! Two of this crate's promises cannot be asserted at runtime, because a program that violates
//! them does not exist: distinct identities are not interchangeable, and a typed entity
//! reference cannot be fabricated from a raw [`EntityId`](mineworld_contracts::EntityId). The
//! cases under `tests/compile_fail/` are the executable form of those promises, and each one's
//! `.stderr` file pins the *reason* the compiler rejects it — not merely that it did.

#[test]
fn distinct_identities_and_unchecked_references_do_not_compile() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs");
}
