use super::*;

#[test]
fn a_qualified_id_splits_at_its_first_dot_and_keeps_a_dotted_slug() {
    let id = QualifiedId::new("core.skill.cooking").expect("qualified");
    assert_eq!((id.namespace(), id.slug()), ("core", "skill.cooking"));
    assert!(QualifiedId::new("cooking").is_err());
    assert!(QualifiedId::new("Core.x").is_err());
    assert!(QualifiedId::new("core.").is_err());
    assert!(ValueSlug::new("80-plus").is_ok());
    assert!(Slug::new("80-plus").is_err());
}

#[test]
fn the_nearest_are_the_three_closest_ties_by_text() {
    let known = ["stature", "status", "statue", "hair-colour"];
    assert_eq!(nearest("statur", known), ["statue", "stature", "status"]);
}
