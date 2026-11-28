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
        ("source: mineworld", "source: elsewhere", "provenance"),
        (
            "sampling: [profile]",
            "sampling: [profile]\nkeeps: [toy.rhythm]",
            "keeps",
        ),
        (
            "sampling: [profile]",
            "sampling: [profile]\nkeeps: [core.a, core.a]",
            "keeps",
        ),
        ("catalogues: [appearance]", "catalogues: []", "catalogues"),
    ] {
        let refusal = Backend::check("person-attributes", &file(&MINEWORLD.replacen(from, to, 1)))
            .expect_err(field);
        assert!(
            refusal.starts_with("BackendMalformed") && refusal.contains(&format!("'{field}'")),
            "{refusal}"
        );
    }
    let mapped = format!("{MINEWORLD}mapping: Not_A_Stem\n");
    assert!(
        Backend::check("p", &file(&mapped))
            .expect_err("mapping")
            .contains("'mapping'")
    );
    assert!(Backend::check("p", &file(&format!("{MINEWORLD}mapping: toy\n"))).is_ok());
}

/// An upstream provenance is checked for its shape only (§5.9, D-h-6): a source, a version, a 40-hex
/// commit and 64-hex file digests.
#[test]
fn an_upstream_provenance_is_checked_for_its_shape() {
    let commit = "0123456789abcdef0123456789abcdef01234567";
    let digest = "ab".repeat(32);
    let upstream = |commit: &str, digest: &str| {
        MINEWORLD.replace(
            "provenance: { source: mineworld }",
            &format!(
                "provenance:\n  source: example.org/upstream\n  version: \"schema 1.0\"\n  commit: {commit}\n  files: {{ .upstream/a.json: {digest} }}"
            ),
        )
    };
    let checked = Backend::check("p", &file(&upstream(commit, &digest))).expect("upstream");
    assert_eq!(
        checked.provenance.to_string(),
        format!("example.org/upstream schema 1.0 at {commit} (1 files)")
    );
    for (bad_commit, bad_digest) in [
        ("0123456", digest.as_str()),
        ("0123456789ABCDEF0123456789ABCDEF01234567", digest.as_str()),
        (commit, "abc"),
    ] {
        let refusal =
            Backend::check("p", &file(&upstream(bad_commit, bad_digest))).expect_err("bad shape");
        assert!(refusal.contains("'provenance'"), "{refusal}");
    }
    let unversioned = MINEWORLD.replace(
        "provenance: { source: mineworld }",
        &format!("provenance: {{ source: example.org/upstream, commit: {commit} }}"),
    );
    assert!(
        Backend::check("p", &file(&unversioned))
            .expect_err("no version")
            .contains("'provenance'")
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
