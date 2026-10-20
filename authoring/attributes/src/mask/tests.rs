use super::{Mask, MaskFile};

fn mask(text: &str) -> Result<Mask, String> {
    let file: MaskFile = serde_saphyr::from_str(text).expect("decodes");
    Mask::check(&file)
}

#[test]
fn an_entry_without_a_why_is_unjustified() {
    let refusal = mask("format: 1\nmask: small\nversion: 1\ninclude:\n  - { id: core.stature }\n")
        .expect_err("no why");
    assert!(refusal.starts_with("MaskEntryUnjustified"), "{refusal}");
    let refusal = mask("format: 1\nmask: small\nversion: 1\ninclude:\n  - { id: core.stature, group: appearance, why: x }\n")
        .expect_err("two selectors");
    assert!(refusal.starts_with("BadMask"), "{refusal}");
}

#[test]
fn the_reader_line_is_the_why_of_an_id_entry() {
    let checked = mask("format: 1\nmask: small\nversion: 1\ninclude:\n  - { id: core.stature, why: \"C, OV: tall\" }\n  - { group: appearance, why: casting }\n")
        .expect("checks");
    let stature = mineworld_id("core.stature");
    assert_eq!(checked.why_of(&stature), Some("C, OV: tall"));
    assert_eq!(checked.why_of(&mineworld_id("core.mass")), None);
    let known = std::collections::BTreeSet::from([mineworld_id("core.mass")]);
    assert_eq!(checked.unknown_ids(&known), vec![&stature]);
}

fn mineworld_id(text: &str) -> crate::id::QualifiedId {
    crate::id::QualifiedId::new(text).expect("an id")
}
