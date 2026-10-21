//! The 3D client's depiction reader agrees with `mineworld_packages::depiction` (S33-d AC-d-10, I-d-12;
//! `docs/DECISIONS.md` ARC-177).
//!
//! The client checks the file's shape, its scope and the names only it knows (texture slugs, tree forms
//! and cards); the Rust validator checks everything else too. So, for every file of the shared corpus
//! and of the default 3D pack's `depictions/`:
//!
//! ```text
//! Rust refuses MALFORMED, SCOPE_MISMATCH or REFERENCE   → the client gives the same code
//! Rust refuses a geometry code (UNCITED … ON_FLOOR)     → the client reads the file (it trusts a validated pack)
//! Rust reads the file                                   → the client reads it, except a `-client-` corpus
//!                                                         file, which the client refuses REFERENCE
//! ```
//!
//! It starts Godot, so it is `#[ignore]`d — reported as ignored, never as passed, without it:
//!
//! ```text
//! cargo test -p mineworld-cli --test client_depiction -- --ignored
//! ```

mod godot2d;
mod headless;
mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

use mineworld_packages::{DepictionCode, PackageError, depiction};

const REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const CLIENT_3D: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../clients/3d-spike");

fn repository(relative: &str) -> PathBuf {
    Path::new(REPOSITORY).join(relative)
}

/// The code the Rust validator gives `path` (with the world's far-field waters when `waters` is given),
/// or `ok`.
fn rust_code(path: &Path, waters: Option<&[String]>) -> String {
    match depiction::read(path, waters) {
        Ok(_) => "ok".to_owned(),
        Err(PackageError::Depiction { code, .. }) => code.as_str().to_owned(),
        Err(other) => panic!("{}: not a depiction refusal: {other}", path.display()),
    }
}

/// What the client must print for a file Rust judges `rust`.
fn client_expected(name: &str, rust: &str) -> String {
    let client_checks = [
        DepictionCode::Malformed,
        DepictionCode::ScopeMismatch,
        DepictionCode::Reference,
    ];
    if name.contains("-client-") {
        return DepictionCode::Reference.as_str().to_owned();
    }
    if client_checks.iter().any(|code| code.as_str() == rust) {
        rust.to_owned()
    } else {
        "ok".to_owned()
    }
}

#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_depiction -- --ignored"]
fn the_client_and_the_validator_agree() {
    let godot = godot2d::godot();
    let imported = Command::new(&godot)
        .args(["--headless", "--path", CLIENT_3D, "--import"])
        .status()
        .expect("godot runs");
    assert!(imported.success(), "the 3D project imports");
    let pack = repository("presentation/mineworld-default/3D");
    let script = Path::new(CLIENT_3D).join("scripts/checks/depiction_check.gd");
    let output = Command::new(&godot)
        .args([
            "--headless",
            "--path",
            CLIENT_3D,
            "--script",
            script.to_str().expect("UTF-8"),
            "--",
            &format!("--pack={}", pack.display()),
        ])
        .output()
        .expect("godot runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        stdout.contains("depiction_check: "),
        "INCONCLUSIVE: the check printed no summary line:\n{stdout}"
    );
    let mut got: Vec<(String, String)> = stdout
        .lines()
        .filter_map(|l| l.strip_prefix("DEPICTION "))
        .map(|rest| {
            let (name, code) = rest.rsplit_once(' ').expect("<name> <code>");
            (name.to_owned(), code.to_owned())
        })
        .collect();
    let mut expected: Vec<(String, String)> = Vec::new();
    let corpus = repository("packages/tests/fixtures/depiction");
    for entry in std::fs::read_dir(&corpus).expect("the corpus lists") {
        let path = entry.expect("an entry").path();
        if path.extension().is_none_or(|e| e != "yaml") {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).expect("a stem");
        let rust = rust_code(&path, None);
        expected.push((stem.to_owned(), client_expected(stem, &rust)));
    }
    let depictions = pack.join(depiction::DEPICTIONS_DIR);
    for entry in std::fs::read_dir(&depictions).expect("depictions/") {
        let path = entry.expect("an entry").path();
        let stem = path.file_stem().and_then(|s| s.to_str()).expect("a stem");
        let rust = rust_code(&path, None);
        expected.push((format!("3d/{stem}"), client_expected(stem, &rust)));
    }
    expected.sort();
    got.sort();
    assert_eq!(
        got, expected,
        "the GDScript reader and the Rust validator disagree (left: GDScript, right: expected)"
    );
    assert!(
        expected
            .iter()
            .any(|(name, code)| name == "3d/lakeside" && code == "ok"),
        "Lakeside's file is read by both"
    );
}
