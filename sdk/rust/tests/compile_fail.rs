//! Guarantees checked by compiling code that must not compile (`DECISIONS.md` `ARC-53`).
//!
//! - **a System Pack without `PACKAGE` does not compile.** Every pack in a build therefore has a
//!   package identity: an anonymous pack is not a pack that `mineworld packs` lists without a name, it
//!   is a program that does not exist. The `.stderr` file pins the reason — the missing item — rather
//!   than merely that the compiler refused.
//! - **a section without a JSON Schema does not compile** (`ARC-90`, QS25-2): the `.stderr` pins the
//!   unmet `JsonSchema` bound on `AuthoredSection::Authored`.

#[test]
fn a_pack_cannot_be_anonymous_or_undocumented() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs");
}
