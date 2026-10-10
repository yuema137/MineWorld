//! Entity Packs through the real binary (`docs/DECISIONS.md` `ARC-71`; step-16 §17.5 ED-1, ED-4, ED-5,
//! ED-6, ED-9): a world uses an Entity Pack's item kinds without copying them and without a rebuild, and
//! every refusal of a required data pack — absent, outside the licence policy, outside its own framework
//! range — is by name with exit 1. `MINEWORLD_PACKS` is removed from every child unless a case sets it.

use std::path::{Path, PathBuf};
use std::process::Command;

const REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

const GOODS: &str = "id: goods\ntype: entity-pack\nversion: 0.1.0\nmineworld: \"^0.1\"\nlicense: MIT\n\
                     authors: [Someone]\n";

/// `mineworld <arguments>` with `MINEWORLD_PACKS` removed: (success, stdout, stderr).
fn mineworld(arguments: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_mineworld"))
        .args(arguments)
        .env_remove("MINEWORLD_PACKS")
        .output()
        .expect("the mineworld binary runs");
    (
        output.status.success(),
        String::from_utf8(output.stdout).expect("UTF-8"),
        String::from_utf8(output.stderr).expect("UTF-8"),
    )
}

fn market_town() -> PathBuf {
    Path::new(REPOSITORY).join("worlds").join("market-town")
}

/// Copies `from` into `to`, byte for byte.
fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("a readable directory") {
        let path = entry.expect("an entry").path();
        let target = to.join(path.file_name().expect("a name"));
        if path.is_dir() {
            copy_tree(&path, &target);
        } else {
            std::fs::copy(&path, &target).expect("copied");
        }
    }
}

/// A scratch directory: ED-1's world `market-town/` and a pack root `root/`, removed when the test ends
/// (DEP-29).
struct Scratch(mineworld_test_support::Scratch);

impl Scratch {
    fn new(name: &str) -> Self {
        let scratch = Self(mineworld_test_support::scratch!(format!(
            "cli-entity-packs-{name}"
        )));
        std::fs::create_dir_all(scratch.0.join("root")).expect("root");
        scratch
    }

    fn path(&self, relative: &str) -> PathBuf {
        relative
            .split('/')
            .fold(self.0.to_path_buf(), |path, part| path.join(part))
    }

    fn text(&self, relative: &str) -> String {
        self.path(relative)
            .to_str()
            .expect("a UTF-8 path")
            .to_owned()
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.path(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("parent directory");
        std::fs::write(path, text).expect("file writes");
    }

    /// ED-1's world: Market Town with `bread` and `coffee` removed from `items:` and `items/`, requiring
    /// `goods: "^0.1"`. Returns the two files' text, for a pack to hold.
    fn moved_town(&self) -> Vec<(&'static str, String)> {
        copy_tree(&market_town(), &self.path("market-town"));
        let manifest = std::fs::read_to_string(self.path("market-town/world.yaml"))
            .expect("world.yaml")
            .replace("\r\n", "\n");
        let mut manifest = manifest;
        let mut files = Vec::new();
        for kind in ["bread", "coffee"] {
            let line = format!("  - {kind}\n");
            assert!(
                manifest.contains(&line),
                "{kind} is one of Market Town's kinds"
            );
            manifest = manifest.replacen(&line, "", 1);
            let file = self.path(&format!("market-town/items/{kind}.yaml"));
            files.push((kind, std::fs::read_to_string(&file).expect("its file")));
            std::fs::remove_file(file).expect("moved out");
        }
        self.write(
            "market-town/world.yaml",
            &format!("{manifest}requires:\n  goods: \"^0.1\"\n"),
        );
        files
    }

    /// ED-1's pack `goods` in `at` (a root), its `pack.yaml` being `pack`, holding `files`.
    fn goods(&self, at: &str, pack: &str, files: &[(&'static str, String)]) {
        self.write(&format!("{at}/goods/pack.yaml"), pack);
        for (kind, text) in files {
            self.write(&format!("{at}/goods/items/{kind}.yaml"), text);
        }
    }

    /// ED-1's world and pack, the pack in `root/`.
    fn fixture(name: &str) -> Self {
        let scratch = Self::new(name);
        let files = scratch.moved_town();
        scratch.goods("root", GOODS, &files);
        scratch
    }
}

/// The lines of a `run` report a person compares two runs by: the history (fact count and fingerprint)
/// and the faults.
fn history(out: &str) -> (String, String) {
    let line = |prefix: &str| {
        out.lines()
            .find(|line| line.starts_with(prefix))
            .unwrap_or_else(|| panic!("no {prefix:?} line in {out}"))
            .to_owned()
    };
    (line("history "), line("faults "))
}

/// ED-1 (AE-4): used without being copied. Resolved in the root, validated with the composed kinds, and
/// run 30 days at seed 7 to the same history as Market Town itself — moving kinds into a pack changes no
/// fact (PD-35).
#[test]
fn a_world_uses_an_entity_pack_without_copying_it() {
    let scratch = Scratch::fixture("used");
    let (world, root) = (scratch.text("market-town"), scratch.text("root"));
    let goods_dir = scratch.path("root/goods").display().to_string();

    let (ok, out, err) = mineworld(&["packs", "resolve", &world, "--packs", &root]);
    assert!(ok, "{err}");
    assert!(
        out.contains("requires   goods \"^0.1\" → entity-pack 0.1.0") && out.contains(&goods_dir),
        "{out}"
    );

    let (ok, out, err) = mineworld(&["validate", &world, "--packs", &root]);
    assert!(ok, "{err}");
    let items = out
        .lines()
        .find(|line| line.starts_with("  items "))
        .unwrap_or_else(|| panic!("an items line: {out}"));
    assert!(
        items.contains("bread") && items.contains("coffee") && items.contains("apple"),
        "the composed kinds: {items}"
    );

    let run = ["run", "--headless", "--seed", "7", "--days", "30"];
    let (ok, moved, err) = mineworld(&[&run[..], &[&world, "--packs", &root]].concat());
    assert!(ok, "{err}");
    let original = market_town().display().to_string();
    let (ok, unmoved, err) = mineworld(&[&run[..], &[original.as_str()]].concat());
    assert!(ok, "{err}");
    let (moved, unmoved) = (history(&moved), history(&unmoved));
    assert_eq!(moved.1, "faults     0", "{moved:?}");
    assert_eq!(moved, unmoved, "the same facts, the same fingerprint");
}

/// ED-4: a required Entity Pack that is absent is refused naming it and where it was searched — and a
/// pack sitting beside the world is not found, because only named roots are searched (QSE-13).
#[test]
fn an_absent_entity_pack_is_refused_naming_where_it_was_searched() {
    let scratch = Scratch::new("absent");
    let files = scratch.moved_town();
    scratch.goods(".", GOODS, &files);
    let world = scratch.text("market-town");

    let (ok, _, err) = mineworld(&["validate", &world]);
    assert!(!ok, "refused");
    assert!(
        err.contains("goods") && err.contains("no pack directory was given"),
        "{err}"
    );

    let root = scratch.text("root");
    let (ok, _, err) = mineworld(&["packs", "resolve", &world, "--packs", &root]);
    assert!(!ok, "refused");
    assert!(
        err.contains("requires: goods") && err.contains(&root),
        "{err}"
    );
}

/// ED-5 and ED-6 for a data pack: a licence outside the policy, and a `mineworld:` range excluding the
/// framework (F-Ed1), each refused by `packs resolve` and `packs validate` naming the pack and the value.
/// A Presentation Pack's range is checked the same way.
#[test]
fn a_data_pack_outside_the_policy_or_the_framework_is_refused_by_name() {
    type Case = (&'static str, String, &'static [&'static str]);
    let cases: [Case; 2] = [
        (
            "licence",
            GOODS.replace("MIT", "GPL-3.0-only"),
            &["goods", "GPL-3.0-only", "MIT, "],
        ),
        (
            "framework",
            GOODS.replace("\"^0.1\"", "\"^9\""),
            &["goods", "\"^9\"", "0.1.0"],
        ),
    ];
    for (case, pack, needles) in cases {
        let scratch = Scratch::new(case);
        let files = scratch.moved_town();
        scratch.goods("root", &pack, &files);
        let (world, root, goods) = (
            scratch.text("market-town"),
            scratch.text("root"),
            scratch.text("root/goods"),
        );
        for arguments in [
            vec!["packs", "resolve", world.as_str(), "--packs", root.as_str()],
            vec!["packs", "validate", goods.as_str()],
        ] {
            let (ok, _, err) = mineworld(&arguments);
            assert!(!ok, "{case}: {arguments:?} refused");
            for needle in needles {
                assert!(err.contains(needle), "{case}: {needle:?} not in {err}");
            }
        }
    }

    let scratch = Scratch::new("presentation-framework");
    scratch.write(
        "root/style-a/pack.yaml",
        &GOODS
            .replace("id: goods", "id: style-a")
            .replace("entity-pack", "presentation-pack")
            .replace("\"^0.1\"", "\"^9\""),
    );
    scratch.write(
        "root/style-a/manifest.yaml",
        "id: style-a\ndimension: [3d]\n",
    );
    let (ok, _, err) = mineworld(&["packs", "validate", &scratch.text("root/style-a")]);
    assert!(!ok, "refused");
    assert!(
        err.contains("style-a") && err.contains("\"^9\"") && err.contains("0.1.0"),
        "{err}"
    );
}

/// `packs validate` of a good Entity Pack prints its identity and its kinds (`ARC-71` point 9).
#[test]
fn packs_validate_lists_an_entity_packs_kinds() {
    let scratch = Scratch::fixture("validate");
    let (ok, out, err) = mineworld(&["packs", "validate", &scratch.text("root/goods")]);
    assert!(ok, "{err}");
    assert!(
        out.contains("  type        entity-pack")
            && out.contains("  items       bread, coffee")
            && out.contains("is a valid entity-pack."),
        "{out}"
    );
}

/// ED-9 (I-E9, §8.2 M-4): no rebuild. A new Entity Pack written into a fresh root after the binary was
/// built is listed, resolved and run with; the binary's length and modification time are unchanged, and
/// nothing here spawns `cargo`. `CARGO_BIN_EXE_mineworld` is the `.exe` on Windows.
#[test]
fn an_entity_pack_is_installed_without_a_rebuild() {
    let binary = Path::new(env!("CARGO_BIN_EXE_mineworld"));
    let before = std::fs::metadata(binary).expect("the binary");
    let (length, modified) = (
        before.len(),
        before.modified().expect("a modification time"),
    );

    let scratch = Scratch::new("no-rebuild");
    let files = scratch.moved_town();
    scratch.goods("fresh", GOODS, &files);
    let (world, root) = (scratch.text("market-town"), scratch.text("fresh"));

    let (ok, out, err) = mineworld(&["packs", "list", "--packs", &root]);
    assert!(ok, "{err}");
    assert!(
        out.lines()
            .any(|line| line.starts_with("entity-pack") && line.contains(" goods ")),
        "{out}"
    );
    let (ok, _, err) = mineworld(&["packs", "resolve", &world, "--packs", &root]);
    assert!(ok, "{err}");
    let (ok, out, err) = mineworld(&[
        "run",
        "--headless",
        "--seed",
        "7",
        "--days",
        "1",
        &world,
        "--packs",
        &root,
    ]);
    assert!(ok, "{err}");
    assert!(out.contains("faults     0"), "{out}");

    let after = std::fs::metadata(binary).expect("the binary");
    assert_eq!(after.len(), length, "the binary was not rebuilt");
    assert_eq!(
        after.modified().expect("a time"),
        modified,
        "the binary was not rebuilt"
    );
}
