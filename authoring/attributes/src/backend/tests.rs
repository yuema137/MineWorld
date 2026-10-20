use std::path::PathBuf;

use super::{Backend, BackendFile, choose};

fn file(text: &str) -> BackendFile {
    serde_saphyr::from_str(text).expect("decodes")
}

const MINEWORLD: &str = "format: 1\nbackend: mineworld\nversion: 1.0.0\nlicense: CC0-1.0\n\
attribution: built-in\ncatalogues: [appearance]\nmasks: { default: core, ships: [core] }\n\
sampling: [profile]\nprovenance: { source: mineworld }\n";

#[test]
fn each_malformed_field_is_named() {
    assert!(Backend::check("person-attributes", &file(MINEWORLD)).is_ok());
    for (from, to, field) in [
        ("sampling: [profile]", "sampling: [log-linear]", "sampling"),
        ("default: core", "default: wide", "masks"),
        ("source: mineworld", "source: matraix", "provenance"),
        ("catalogues: [appearance]", "catalogues: []", "catalogues"),
    ] {
        let refusal = Backend::check("person-attributes", &file(&MINEWORLD.replacen(from, to, 1)))
            .expect_err(field);
        assert!(
            refusal.starts_with("BackendMalformed") && refusal.contains(&format!("'{field}'")),
            "{refusal}"
        );
    }
    let mapped = format!("{MINEWORLD}mapping: {{ a: b }}\n");
    assert!(
        Backend::check("p", &file(&mapped))
            .expect_err("mapping")
            .contains("'mapping'")
    );
}

#[test]
fn a_world_names_its_backend_or_gets_the_only_one() {
    let one = Backend::check("person-attributes", &file(MINEWORLD)).expect("checks");
    let mut toy = one.clone();
    toy.id = "toy".to_owned();
    toy.pack = "toy-backend".to_owned();
    let both = vec![(PathBuf::from("a"), one.clone()), (PathBuf::from("b"), toy)];
    assert_eq!(
        choose(&both[..1], None)
            .expect("one")
            .map(|b| b.id.as_str()),
        Some("mineworld")
    );
    assert_eq!(choose(&[], None).expect("none"), None);
    assert!(
        choose(&both, None)
            .expect_err("two")
            .starts_with("AmbiguousBackend")
    );
    assert_eq!(
        choose(&both, Some("toy"))
            .expect("named")
            .map(|b| b.id.as_str()),
        Some("toy")
    );
    assert!(
        choose(&both, Some("other"))
            .expect_err("unknown")
            .starts_with("UnknownBackend")
    );
}

#[test]
fn a_reader_line_begins_with_reader_codes() {
    assert!(super::is_reader_line("C, OV: height in both clients; tall"));
    assert!(super::is_reader_line("S31: notices"));
    assert!(!super::is_reader_line("casting-only use"));
    assert!(!super::is_reader_line("X: unknown code"));
    assert!(!super::is_reader_line("C: "));
}
