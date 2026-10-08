//! `pack.yaml` and the directories `mineworld packs` reads: every refusal names the file and what is
//! wrong (step-16 §14.4 EA-4; `DECISIONS.md` `ARC-53`).

use std::path::{Path, PathBuf};

use mineworld_packages::{DataPack, PackType, PackageError, check_style_manifest, packs_in};

/// A directory under Cargo's per-test scratch root, removed when dropped (overall ruling 10).
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("packages-{name}"));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("scratch directory");
        Self(path)
    }

    fn write(&self, relative: &str, text: &str) -> PathBuf {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("parent directory");
        std::fs::write(&path, text).expect("file writes");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const GOOD: &str = "id: style-pack\ntype: presentation-pack\nversion: 0.1.0\nmineworld: \"^0.1\"\n\
                    license: MIT\nauthors: [Yue Ma]\n";

fn read(text: &str) -> Result<mineworld_packages::Identity, PackageError> {
    let scratch = Scratch::new(&format!("read-{:016x}", fnv(text)));
    scratch.write("pack.yaml", text);
    mineworld_packages::read_pack_file(&scratch.0)
}

/// A stable short name per case, so cases never share a scratch directory.
fn fnv(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

#[test]
fn a_well_formed_pack_file_is_one_identity() {
    let identity = read(GOOD).expect("reads");
    assert_eq!(identity.id.as_str(), "style-pack");
    assert_eq!(identity.kind, PackType::PresentationPack);
    assert_eq!(identity.version.to_string(), "0.1.0");
    assert_eq!(identity.license.as_str(), "MIT");
    assert_eq!(identity.authors, ["Yue Ma"]);
    assert_eq!(identity.mineworld.expect("stated").to_string(), "^0.1");
}

/// Every refusal of EA-4 that `pack.yaml` itself can carry: the file's path and the offending field
/// or value are both in the message.
#[test]
fn every_bad_pack_file_is_refused_naming_what_is_wrong() {
    let cases: [(&str, String, &[&str]); 11] = [
        (
            "malformed semver",
            GOOD.replace("version: 0.1.0", "version: 0.1"),
            &["'0.1' is not a semver version"],
        ),
        (
            "unknown licence",
            GOOD.replace("license: MIT", "license: NOT-A-LICENCE"),
            &["'NOT-A-LICENCE' is not an SPDX licence expression"],
        ),
        (
            "missing field",
            GOOD.replace("license: MIT\n", ""),
            &["license"],
        ),
        (
            "unknown field",
            format!("{GOOD}colour: blue\n"),
            &["colour"],
        ),
        (
            "dependencies, not yet resolved",
            format!("{GOOD}dependencies: {{ other: \"^0.1\" }}\n"),
            &["dependencies"],
        ),
        (
            "a world's type",
            GOOD.replace("presentation-pack", "world-pack"),
            &["type world-pack", "world.yaml"],
        ),
        (
            "a code pack's type",
            GOOD.replace("presentation-pack", "system-pack"),
            &["type system-pack", "Cargo.toml"],
        ),
        (
            "an entity pack, before E-d",
            GOOD.replace("presentation-pack", "entity-pack"),
            &["type entity-pack", "E-d"],
        ),
        (
            "an asset pack",
            GOOD.replace("presentation-pack", "asset-pack"),
            &["'asset-pack' is not a pack type", "MVP-0"],
        ),
        (
            "an id outside the rule",
            GOOD.replace("id: style-pack", "id: Style_Pack"),
            &["'Style_Pack' is not a pack id"],
        ),
        ("no author", GOOD.replace("[Yue Ma]", "[]"), &["no author"]),
    ];
    for (case, text, needles) in cases {
        let refusal = read(&text).expect_err(case).to_string();
        assert!(
            refusal.contains("pack.yaml"),
            "{case}: no file named in {refusal}"
        );
        for needle in needles {
            assert!(
                refusal.contains(needle),
                "{case}: {needle:?} not in {refusal}"
            );
        }
    }
}

#[test]
fn a_style_manifest_needs_an_id_and_a_dimension() {
    let scratch = Scratch::new("style");
    scratch.write(
        "manifest.yaml",
        "id: a-style\nname: A\ndimension: [3d]\nlighting: soft\n",
    );
    check_style_manifest(&scratch.0).expect("other fields are the style's own");
    for (text, needle) in [
        ("name: A\ndimension: [3d]\n", "id"),
        ("id: a-style\n", "dimension"),
        ("id: a-style\ndimension: []\n", "dimension"),
    ] {
        scratch.write("manifest.yaml", text);
        let refusal = check_style_manifest(&scratch.0)
            .expect_err(text)
            .to_string();
        assert!(
            refusal.contains("manifest.yaml") && refusal.contains(needle),
            "{refusal}"
        );
    }
}

/// `packs_in` reads only the first level of the directory it was given, by manifest: a directory with
/// neither manifest is not a pack, one with both is refused, and a root that is a pack is refused.
#[test]
fn a_directory_of_packs_is_read_by_manifest_and_nothing_else() {
    let scratch = Scratch::new("root");
    scratch.write("b-world/world.yaml", "");
    scratch.write("a-style/pack.yaml", "");
    scratch.write("LICENSES/SOMETHING.txt", "");
    scratch.write("README.md", "");
    scratch.write("a-style/nested/world.yaml", "");
    let found = packs_in(&scratch.0).expect("reads");
    assert_eq!(
        found,
        [
            DataPack::PackFile(scratch.0.join("a-style")),
            DataPack::World(scratch.0.join("b-world")),
        ]
    );

    scratch.write("b-world/pack.yaml", "");
    let refusal = packs_in(&scratch.0).expect_err("two manifests").to_string();
    assert!(
        refusal.contains("b-world") && refusal.contains("both"),
        "{refusal}"
    );

    let refusal = packs_in(&scratch.0.join("a-style"))
        .expect_err("a pack is not a root")
        .to_string();
    assert!(
        refusal.contains("is a pack") && refusal.contains("packs validate"),
        "{refusal}"
    );
}
