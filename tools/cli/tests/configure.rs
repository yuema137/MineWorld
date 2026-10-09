//! IA-3 and IA-4 (b) at the binary: `configure:` refused by the real `mineworld validate`, and every
//! host that resumes or verifies a save checks the World Pack's configuration first
//! (`DECISIONS.md` `ARC-61` item 7).
//!
//! That the check *refuses* a changed configuration through the real binary needs a configurable pack
//! installed, which this build has none of: it is shown by the canary (IA-10), and the comparator itself
//! by worldpack's own tests (IA-4 a). What this file holds is that no resuming host can skip the check —
//! structurally, because only a structural test can hold an absence — and that the existing restart
//! tests (`restart.rs`, `run_restart.rs` and the yard's restart test, unedited) still resume.

use std::path::Path;
use std::process::{Command, Output};

/// The source of one of this binary's modules.
fn source(file: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The text of the function whose signature begins with `signature`: from it to the first line that
/// closes a function at its indentation.
fn function<'s>(source: &'s str, signature: &str) -> &'s str {
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

/// In the function, the configuration check comes before `call`, and both are there.
fn checks_before(file: &str, signature: &str, call: &str) {
    let source = source(file);
    let text = function(&source, signature);
    let check = text
        .find("check_configuration(")
        .unwrap_or_else(|| panic!("{file} `{signature}` never checks the configuration"));
    let resumes = text
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

/// A scratch World Pack for `validate`: `mineworld-test-support`'s scratch, named as the pack and
/// removed when the test ends (`DEP-29`).
struct Scratch(mineworld_test_support::Scratch);

impl Scratch {
    fn new(id: &str, configure: &str) -> Self {
        let scratch = Self(mineworld_test_support::scratch!(empty id));
        for directory in ["people", "places", "configure"] {
            std::fs::create_dir_all(scratch.0.path().join(directory))
                .expect("a writable temporary directory");
        }
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
        std::fs::write(self.0.path().join(relative), contents).expect("a writable scratch file");
    }

    fn validate(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_mineworld"))
            .arg("validate")
            .arg(self.0.path())
            .output()
            .expect("the binary runs")
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
            "cli-configure-packages",
            "configure:\n  - packages\n",
            Some("configure/packages.yaml"),
            "the world's licence policy is not valid",
        ),
        (
            "cli-configure-not-enabled",
            "configure:\n  - schedule\n",
            None,
            "'schedule', which this world does not enable",
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

/// IB-9 through the real binary: a world that narrows its licence policy to Apache-2.0 is refused by
/// both `validate` and `packs validate <world>`, naming a bundled MIT pack and the allowed list; the
/// same world allowing MIT passes both. `packs validate` judges a World Pack by its own policy.
#[test]
fn a_world_licence_policy_governs_validate_and_packs_validate() {
    let run = |args: &[&std::ffi::OsStr]| {
        Command::new(env!("CARGO_BIN_EXE_mineworld"))
            .args(args)
            .output()
            .expect("the binary runs")
    };
    for (allowed, passes) in [("Apache-2.0", false), ("MIT", true)] {
        let id = format!("cli-packages-{}", allowed.to_lowercase().replace('.', "-"));
        let scratch = Scratch::new(&id, "configure:\n  - packages\n");
        scratch.write(
            "world.yaml",
            &format!(
                "world:\n  id: {id}\n  name: A Scratch World\n  version: 0.1.0\n  license: MIT OR \
                 Apache-2.0\nmineworld: \"^0.1\"\nsystems:\n  - presence\n  - movement\nplaces:\n  - square\n\
                 population:\n  - ada\nconfigure:\n  - packages\n"
            ),
        );
        scratch.write(
            "configure/packages.yaml",
            &format!("allowed: [{allowed}]\n"),
        );
        let path = scratch.0.path().as_os_str();
        for output in [
            run(&["validate".as_ref(), path]),
            run(&["packs".as_ref(), "validate".as_ref(), path]),
        ] {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_eq!(output.status.success(), passes, "{allowed}: {stderr}");
            if !passes {
                assert!(
                    stderr.contains("MIT")
                        && stderr.contains("Apache-2.0")
                        && (stderr.contains("presence") || stderr.contains("movement")),
                    "names a bundled MIT pack, its licence and the allowed list: {stderr}"
                );
            }
        }
    }
}
