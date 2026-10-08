//! A world's `requires:` through the real binary: met in the pack roots given by `--packs` or
//! `MINEWORLD_PACKS` and nowhere else, honoured by every command that reads a world, and every failure
//! refused by name with exit 1 (`docs/DECISIONS.md` `ARC-54`, `ARC-55`; step-16 §15.4 EB-1, EB-2,
//! EB-3, EB-5, EB-8). The rule that an enabled third-party system must be required has no third-party
//! code pack in this build to exercise it until S16's E-c; it is held by `packages/tests/resolve.rs`.

use std::path::{Path, PathBuf};
use std::process::Command;

use mineworld_worldpack::catalog::AVAILABLE;

const REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// `mineworld <arguments>` with `MINEWORLD_PACKS` set to `environment` or removed: (success, stdout,
/// stderr). The variable is always controlled, so a developer's own setting cannot leak in.
fn mineworld_with(arguments: &[&str], environment: Option<&str>) -> (bool, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mineworld"));
    command.args(arguments).env_remove("MINEWORLD_PACKS");
    if let Some(value) = environment {
        command.env("MINEWORLD_PACKS", value);
    }
    let output = command.output().expect("the mineworld binary runs");
    (
        output.status.success(),
        String::from_utf8(output.stdout).expect("UTF-8"),
        String::from_utf8(output.stderr).expect("UTF-8"),
    )
}

fn mineworld(arguments: &[&str]) -> (bool, String, String) {
    mineworld_with(arguments, None)
}

/// A scratch directory: a world `the-world` beside a pack root `root/`, removed when dropped.
struct Scratch(PathBuf);

const STYLE: &str = "type: presentation-pack\nversion: 0.1.0\nmineworld: \"^0.1\"\nlicense: MIT\n\
                     authors: [Someone]\n";

impl Scratch {
    /// A one-place, two-seat world stating `identity` (lines under `world:`) and `top` (top-level
    /// lines), with an empty `root/` beside it.
    fn new(name: &str, identity: &str, top: &str) -> Self {
        let base = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cli-requirements-{name}"));
        let _ = std::fs::remove_dir_all(&base);
        let scratch = Self(base);
        scratch.write(
            "the-world/world.yaml",
            &format!(
                "world:\n  id: the-world\n  name: Requirements\n{identity}{top}systems: [presence, \
                 movement, conversation]\nplaces: [home]\npopulation: [first, second]\n\
                 seats: [first, second]\n"
            ),
        );
        scratch.write("the-world/places/home.yaml", "tags: [home]\n");
        for person in ["first", "second"] {
            scratch.write(
                &format!("the-world/people/{person}.yaml"),
                "tags: [resident]\nlocation:\n  place: home\n",
            );
        }
        std::fs::create_dir_all(scratch.0.join("root")).expect("root");
        scratch
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("parent directory");
        std::fs::write(path, text).expect("file writes");
    }

    /// A presentation pack `id` at `version` with `license`, in `root/` (or wherever `at` says).
    fn style(&self, at: &str, id: &str, version: &str, license: &str) {
        let text = format!(
            "id: {id}\n{}",
            STYLE.replace("0.1.0", version).replace("MIT", license)
        );
        self.write(&format!("{at}/{id}/pack.yaml"), &text);
    }

    fn path(&self, relative: &str) -> String {
        self.0
            .join(relative)
            .to_str()
            .expect("a UTF-8 path")
            .to_owned()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const REQUIRES_STYLE: &str = "requires:\n  style-a: \"^0.1\"\n";

/// EB-1: a requirement met in a root, through every command that reads a world — and a save resumed
/// without that root is refused at read, because resolution, not the save, holds the composition
/// (EB-8).
#[test]
fn a_world_whose_requirement_is_met_runs_through_every_command() {
    let scratch = Scratch::new("met", "", REQUIRES_STYLE);
    scratch.style("root", "style-a", "0.1.4", "MIT");
    let (world, root, save) = (
        scratch.path("the-world"),
        scratch.path("root"),
        scratch.path("save"),
    );

    let (ok, out, err) = mineworld(&["packs", "resolve", &world, "--packs", &root]);
    assert!(ok, "{err}");
    assert!(
        out.contains("requires   style-a \"^0.1\" → presentation-pack 0.1.4")
            && out.contains("framework  0.1.0")
            && out.contains("system     presence → mineworld-presence 0.1.0 (bundled)"),
        "{out}"
    );

    let (ok, out, err) = mineworld(&["validate", &world, "--packs", &root]);
    assert!(ok, "{err}");
    assert!(
        out.contains("  requires   style-a \"^0.1\" → 0.1.4"),
        "{out}"
    );

    let run = [
        "run",
        &world,
        "--headless",
        "--seed",
        "3",
        "--days",
        "1",
        "--save",
        &save,
    ];
    let (ok, _, err) = mineworld(&[&run[..], &["--packs", &root]].concat());
    assert!(ok, "{err}");
    let (ok, _, err) = mineworld(&["replay", &world, "--save", &save, "--packs", &root]);
    assert!(ok, "{err}");
    let (ok, _, err) = mineworld(&[
        "biography",
        &world,
        "--save",
        &save,
        "--person",
        "first",
        "--packs",
        &root,
    ]);
    assert!(ok, "{err}");

    let (ok, _, err) = mineworld(&["replay", &world, "--save", &save]);
    assert!(!ok, "resumed without the root");
    assert!(
        err.contains("requires: style-a") && err.contains("no pack directory was given"),
        "{err}"
    );
}

/// EB-1 with the repository's own Presentation Pack, found through `--packs`.
#[test]
fn the_default_presentation_pack_can_be_required() {
    let scratch = Scratch::new(
        "default-3d",
        "",
        "requires:\n  mineworld-default-3d: \"^0.1\"\n",
    );
    let presentation = format!("{REPOSITORY}/presentation/mineworld-default");
    let (ok, out, err) = mineworld(&[
        "packs",
        "resolve",
        &scratch.path("the-world"),
        "--packs",
        &presentation,
    ]);
    assert!(ok, "{err}");
    assert!(
        out.contains("requires   mineworld-default-3d \"^0.1\""),
        "{out}"
    );
}

/// EB-2: each refusal through the real binary, by name, exit 1.
#[test]
fn every_unmet_requirement_is_refused_by_name() {
    type Setup = fn(&Scratch);
    let cases: [(&str, &str, &str, Setup, &[&str]); 7] = [
        (
            "absent",
            "",
            "requires:\n  nowhere: \"^0.1\"\n",
            |_| {},
            &["requires: nowhere", "root"],
        ),
        (
            "out-of-range",
            "",
            "requires:\n  style-a: \"^0.2\"\n",
            |s| s.style("root", "style-a", "0.1.0", "MIT"),
            &["style-a \"^0.2\"", "found is 0.1.0"],
        ),
        (
            "wrong-type",
            "",
            "requires:\n  other-world: \"^0.1\"\n",
            |s| {
                s.write(
                    "root/other-world/world.yaml",
                    "world:\n  id: other-world\n  name: O\n  version: 0.1.0\n  license: MIT\n\
                     mineworld: \"^0.1\"\n",
                );
            },
            &["other-world is a world-pack"],
        ),
        (
            "bundled",
            "",
            "requires:\n  mineworld-presence: \"^0.1\"\n",
            |_| {},
            &["mineworld-presence is bundled"],
        ),
        (
            "world-licence",
            "  license: GPL-3.0-only\n",
            "",
            |_| {},
            &[
                "the-world's licence \"GPL-3.0-only\"",
                "GPL-3.0-only",
                "CC0-1.0",
            ],
        ),
        (
            "pack-licence",
            "",
            REQUIRES_STYLE,
            |s| s.style("root", "style-a", "0.1.0", "CC-BY-SA-4.0"),
            &["style-a's licence \"CC-BY-SA-4.0\""],
        ),
        (
            "duplicate",
            "",
            REQUIRES_STYLE,
            |s| {
                s.style("root", "style-a", "0.1.0", "MIT");
                s.style("root-2", "style-a", "0.1.0", "MIT");
            },
            &[
                "two packs have the id style-a",
                "root/style-a",
                "root-2/style-a",
            ],
        ),
    ];
    for (name, identity, top, setup, needles) in cases {
        let scratch = Scratch::new(name, identity, top);
        setup(&scratch);
        std::fs::create_dir_all(scratch.0.join("root-2")).expect("second root");
        let (ok, _, err) = mineworld(&[
            "validate",
            &scratch.path("the-world"),
            "--packs",
            &scratch.path("root"),
            "--packs",
            &scratch.path("root-2"),
        ]);
        assert!(!ok, "{name}: accepted");
        for needle in needles {
            assert!(err.contains(needle), "{name}: {needle:?} not in {err}");
        }
    }
}

/// EB-3: nothing implicit. The pack that would meet the requirement sits right beside the world, and
/// is not found unless a root names a directory holding it; `MINEWORLD_PACKS` alone names one; a root
/// that does not exist is refused naming where it came from.
///
/// The world's own parent is not used as that root: it holds the world itself, which would then be a
/// pack of the root and must state its own package fields (ARC-54 point 4, rule 1).
#[test]
fn only_the_named_roots_are_searched() {
    let scratch = Scratch::new("implicit", "", REQUIRES_STYLE);
    scratch.style(".", "style-a", "0.1.0", "MIT");
    scratch.style("near", "style-a", "0.1.0", "MIT");
    let world = scratch.path("the-world");

    let (ok, _, err) = mineworld(&["validate", &world]);
    assert!(!ok, "found a pack nobody named");
    assert!(err.contains("no pack directory was given"), "{err}");

    let near = scratch.path("near");
    let (ok, _, err) = mineworld_with(&["validate", &world], Some(&near));
    assert!(ok, "MINEWORLD_PACKS names it: {err}");

    let missing = scratch.path("missing");
    let (ok, _, err) = mineworld(&["validate", &world, "--packs", &missing]);
    assert!(
        !ok && err.contains("missing") && err.contains("--packs"),
        "{err}"
    );
    let (ok, _, err) = mineworld_with(&["validate", &world], Some(&missing));
    assert!(!ok && err.contains("MINEWORLD_PACKS"), "{err}");
}

/// EB-5: every installed System Pack is bundled, and a world that requires nothing resolves with no
/// root — located from the installed set, never from a list written here.
#[test]
fn an_existing_world_resolves_with_every_system_bundled() {
    let social = format!("{REPOSITORY}/worlds/social-cafe");
    let (ok, out, err) = mineworld(&["packs", "resolve", &social]);
    assert!(ok, "{err}");
    assert!(
        out.contains("system     presence → mineworld-presence"),
        "presence is located first: {out}"
    );
    let systems: Vec<&str> = out
        .lines()
        .filter(|l| l.starts_with("  system     "))
        .collect();
    assert!(!systems.is_empty() && systems.len() <= AVAILABLE.len());
    for line in &systems {
        assert!(line.ends_with("(bundled)"), "{line}");
    }
    assert!(!out.contains("  requires"), "{out}");
}

/// FQ-b1: a world `mineworld create` writes is a valid pack.
#[test]
fn a_created_world_is_a_valid_pack() {
    let scratch = Scratch::new("created", "", "");
    let town = scratch.path("my-town");
    let (ok, _, err) = mineworld(&["create", &town]);
    assert!(ok, "{err}");
    let (ok, out, err) = mineworld(&["packs", "validate", &town]);
    assert!(ok, "{err}");
    assert!(
        out.contains("license     MIT") && out.contains("mineworld   ^0.1"),
        "{out}"
    );
}
