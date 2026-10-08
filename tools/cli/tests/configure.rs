//! IA-3 and IA-4 (b) at the binary: `configure:` refused by the real `mineworld validate`, and every
//! host that resumes or verifies a save checks the World Pack's configuration first
//! (`DECISIONS.md` `ARC-61` item 7).
//!
//! That the check *refuses* a changed configuration through the real binary needs a configurable pack
//! installed, which this build has none of: it is shown by the canary (IA-10), and the comparator itself
//! by worldpack's own tests (IA-4 a). What this file holds is that no resuming host can skip the check —
//! structurally, because only a structural test can hold an absence — and that the existing restart
//! tests (`restart.rs`, `run_restart.rs`, `bodies_yard_restart.rs`, unedited) still resume.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The source of one of this binary's modules.
fn source(file: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The body of the function whose signature begins with `signature`: from it to the first line that
/// closes a function at its indentation.
fn body<'s>(source: &'s str, signature: &str) -> &'s str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("no function `{signature}`"));
    let indent = source[..start].len() - source[..start].trim_end_matches(' ').len();
    let close = format!("\n{}}}\n", " ".repeat(indent));
    let end = source[start..]
        .find(&close)
        .unwrap_or_else(|| panic!("`{signature}` does not close"));
    &source[start..start + end]
}

/// In `body`, the configuration check comes before `call`, and both are there.
fn checks_before(file: &str, signature: &str, call: &str) {
    let source = source(file);
    let body = body(&source, signature);
    let check = body
        .find("check_configuration(")
        .unwrap_or_else(|| panic!("{file} `{signature}` never checks the configuration"));
    let resumes = body
        .find(call)
        .unwrap_or_else(|| panic!("{file} `{signature}` no longer calls {call}"));
    assert!(
        check < resumes,
        "{file} `{signature}` checks the configuration after {call}"
    );
}

#[test]
fn every_resuming_host_checks_the_configuration_before_it_resumes_or_verifies() {
    checks_before(
        "run.rs",
        "fn new(pack: &WorldPack, save: Option<&Path>)",
        "PersistentWorld::resume(",
    );
    checks_before("main.rs", "fn persisted(", "PersistentWorld::resume(");
    checks_before("main.rs", "fn replay(", "verify(&backend");
}

/// A scratch World Pack for `validate`, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(id: &str, configure: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(id);
        let _ = std::fs::remove_dir_all(&root);
        for directory in ["people", "places", "configure"] {
            std::fs::create_dir_all(root.join(directory)).expect("a writable temporary directory");
        }
        let scratch = Self(root);
        scratch.write(
            "world.yaml",
            &format!(
                "world:\n  id: {id}\n  name: A Scratch World\nsystems:\n  - presence\n  - movement\n\
                 places:\n  - square\npopulation:\n  - ada\n{configure}"
            ),
        );
        scratch.write("places/square.yaml", "tags: [square]\n");
        scratch.write(
            "people/ada.yaml",
            "tags: [walker]\nlocation:\n  place: square\n",
        );
        scratch
    }

    fn write(&self, relative: &str, contents: &str) {
        std::fs::write(self.0.join(relative), contents).expect("a writable scratch file");
    }

    fn validate(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_mineworld"))
            .arg("validate")
            .arg(&self.0)
            .output()
            .expect("the binary runs")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `validate` refuses each reachable `configure:` mistake by name, exits non-zero, and names the file.
#[test]
fn validate_refuses_each_configuration_mistake_by_name() {
    let cases = [
        (
            "cli-configure-unknown",
            "configure:\n  - weather\n",
            None,
            "'weather', which is not a system this build provides",
        ),
        (
            "cli-configure-reserved",
            "configure:\n  - classes\n",
            None,
            "reserved for",
        ),
        (
            "cli-configure-not-enabled",
            "configure:\n  - conversation\n",
            None,
            "'conversation', which this world does not enable",
        ),
        (
            "cli-configure-not-configurable",
            "configure:\n  - movement\n",
            None,
            "'movement', which takes no configuration",
        ),
        (
            "cli-configure-twice",
            "configure:\n  - movement\n  - movement\n",
            None,
            "lists 'movement' twice",
        ),
        (
            "cli-configure-undeclared",
            "",
            Some("configure/movement.yaml"),
            "is not listed in world.yaml's configure:",
        ),
    ];
    for (id, configure, stray, expected) in cases {
        let scratch = Scratch::new(id, configure);
        if let Some(file) = stray {
            scratch.write(file, "{}\n");
        }
        let output = scratch.validate();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{id}: refused, exit non-zero");
        assert!(
            stderr.contains(expected),
            "{id}: names the mistake: {stderr}"
        );
        assert!(stderr.contains(id), "{id}: names the file: {stderr}");
    }
}
