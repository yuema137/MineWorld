//! The decision list through the real binary (`pr-dl-a-decision-list.md` C5; AC-DLa-1, -3, -5, -7, -11):
//!
//! ```text
//! no list           run's header and summary equal the base's, captured before DL-a changed any code
//! --seat-model      the header carries the decisions clause; lists that resolve alike print alike
//! refusals          --decisions, --seat-model, a pack's bad decisions.yaml — by file, line, column
//! server            --decisions without --town refused; with --town the town: statement carries it
//! decisions --json  the cognition process's document: models, memory per seat, key order (golden)
//! ```

mod support;

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use support::{INVITE, PACK, Server};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/decisions");

/// `mineworld <arguments>`: exit status, stdout, stderr.
fn mineworld(arguments: &[&str]) -> (bool, String, String) {
    let Output {
        status,
        stdout,
        stderr,
    } = Command::new(env!("CARGO_BIN_EXE_mineworld"))
        .args(arguments)
        .stdin(Stdio::null())
        .output()
        .expect("the mineworld binary runs");
    (
        status.success(),
        String::from_utf8(stdout).expect("UTF-8"),
        String::from_utf8(stderr).expect("UTF-8"),
    )
}

/// Everything `run` printed before its `wall` line: a function of the pack, the seed, the age, the list
/// and the code.
fn without_wall(stdout: &str) -> String {
    stdout
        .lines()
        .filter(|line| !line.starts_with("wall "))
        .map(|line| format!("{line}\n"))
        .collect()
}

/// The header's decisions clause: `; decisions <digest> (…)…`, or `None`.
fn clause(stdout: &str) -> Option<String> {
    let header = stdout.lines().next()?;
    header
        .find("; decisions ")
        .map(|at| header[at..].to_owned())
}

fn digest_of(clause: &str) -> String {
    clause
        .trim_start_matches("; decisions ")
        .split(' ')
        .next()
        .expect("a digest")
        .to_owned()
}

/// A copy of Social Café in scratch (its directory named as the pack), with `world.yaml` extended and
/// extra files written; removed when dropped.
struct Cafe {
    root: PathBuf,
    _scratch: mineworld_test_support::Scratch,
}

impl Cafe {
    fn new(name: &str, world_yaml: &str, files: &[(&str, &str)]) -> Self {
        let scratch = mineworld_test_support::scratch!(empty name).within("social-cafe");
        let root = scratch.path().to_path_buf();
        copy_dir(Path::new(PACK), &root);
        let manifest = root.join("world.yaml");
        let text = std::fs::read_to_string(&manifest).expect("world.yaml reads");
        std::fs::write(&manifest, format!("{text}{world_yaml}")).expect("world.yaml writes");
        for (relative, contents) in files {
            let path = root.join(relative);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
            std::fs::write(path, contents).expect("writes");
        }
        Self {
            root,
            _scratch: scratch,
        }
    }

    fn path(&self) -> &str {
        self.root.to_str().expect("UTF-8")
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("the pack lists") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("a copy");
        }
    }
}

/// A decision list written to scratch, for `--decisions`.
fn list_file(name: &str, contents: &str) -> (mineworld_test_support::Scratch, String) {
    let scratch = mineworld_test_support::scratch!(empty name);
    let path = scratch.path().join("decisions.yaml");
    std::fs::write(&path, contents).expect("writes");
    let text = path.to_str().expect("UTF-8").to_owned();
    (scratch, text)
}

/// AC-DLa-1 (= AC-DL-1): with no decisions: key and no flag, everything `run` prints before `wall` is
/// what the base binary printed — header line included, byte for byte. The golden was captured from
/// `origin/main @ 6306977` before any DL-a code existed (test rules §25).
#[test]
fn with_no_decision_list_run_prints_what_the_base_printed() {
    let (ok, stdout, stderr) =
        mineworld(&["run", PACK, "--headless", "--seed", "7", "--days", "30"]);
    assert!(ok, "{stderr}");
    let golden =
        std::fs::read_to_string(Path::new(FIXTURES).join("social-cafe-seed-7-30-days.txt"))
            .expect("the golden reads");
    assert_eq!(without_wall(&stdout), golden);
    assert_eq!(clause(&stdout), None, "no list in effect, no clause");
}

/// AC-DLa-3, -5, -7: a list in effect prints its digest; three lists that resolve every seat to `rule` —
/// a flag, an operator file, the pack's own file — print the same digest and run the same world as no
/// list; removing the pack's key runs AC-DLa-1's world.
#[test]
fn lists_that_resolve_alike_print_alike_and_run_the_same_world() {
    let days = ["--headless", "--seed", "7", "--days", "3"];
    let run = |pack: &str, extra: &[&str]| {
        let mut arguments = vec!["run", pack];
        arguments.extend_from_slice(&days);
        arguments.extend_from_slice(extra);
        let (ok, stdout, stderr) = mineworld(&arguments);
        assert!(ok, "{arguments:?}: {stderr}");
        stdout
    };
    let plain = run(PACK, &[]);
    let flagged = run(PACK, &["--seat-model", "alice=rule"]);
    let (_file, file) = list_file("decisions-operator", "default: rule\n");
    let operated = run(PACK, &["--decisions", &file]);
    let cafe = Cafe::new(
        "decisions-pack",
        "decisions: decisions.yaml\n",
        &[(
            "decisions.yaml",
            "models:\n  rule: {}\nassign:\n  - { seat: alice, model: rule }\n",
        )],
    );
    let packed = run(cafe.path(), &[]);

    let flag_clause = clause(&flagged).expect("a flag puts a list in effect");
    assert!(flag_clause.ends_with(" (11 rule)"), "{flag_clause}");
    let digest = digest_of(&flag_clause);
    assert_eq!(digest.len(), 16, "{flag_clause}");
    for (what, stdout) in [
        ("--decisions", &operated),
        ("the pack's decisions.yaml", &packed),
    ] {
        assert_eq!(
            clause(stdout).map(|clause| digest_of(&clause)),
            Some(digest.clone()),
            "{what}"
        );
    }
    // Everything after the header is the same world: every seat is still `rule`.
    let body = |stdout: &str| {
        without_wall(stdout)
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n")
    };
    for stdout in [&flagged, &operated, &packed] {
        assert_eq!(body(stdout), body(&plain));
    }
    // A memory: section enters the digest; a changed memory parameter changes it.
    let digest_with = |memory: &str| {
        let (_scratch, file) = list_file("decisions-memory", memory);
        let (ok, stdout, stderr) = mineworld(&["decisions", PACK, "--decisions", &file, "--json"]);
        assert!(ok, "{stderr}");
        let document: serde_json::Value = serde_json::from_str(&stdout).expect("JSON");
        document["digest"].as_str().expect("a digest").to_owned()
    };
    let three = digest_with("memory:\n  default: { policy: decay, parameters: { days: 3 } }\n");
    assert_ne!(three, digest);
    assert_ne!(
        three,
        digest_with("memory:\n  default: { policy: decay, parameters: { days: 4 } }\n")
    );
}

/// AC-DLa-2/-5 through the binary: each refusal names the file, line and column, or the flag.
#[test]
fn refusals_name_the_file_line_and_column_or_the_flag() {
    let (_file, file) = list_file("decisions-bad", "models:\n  mystery: {}\n");
    let (ok, _, stderr) = mineworld(&[
        "run",
        PACK,
        "--headless",
        "--seed",
        "7",
        "--days",
        "1",
        "--decisions",
        &file,
    ]);
    assert!(!ok);
    assert!(
        stderr.contains(&format!(
            "[mineworld] --decisions: {file}:2:3: models: 'mystery' is not a model this build \
             installs; it installs: rule"
        )),
        "{stderr}"
    );
    let (ok, _, stderr) = mineworld(&[
        "run",
        PACK,
        "--headless",
        "--seed",
        "7",
        "--days",
        "1",
        "--seat-model",
        "otto=rule",
    ]);
    assert!(!ok);
    assert!(
        stderr.contains("[mineworld] --seat-model otto=rule: this world offers no such seat"),
        "{stderr}"
    );
    let (ok, _, stderr) = mineworld(&[
        "run",
        PACK,
        "--headless",
        "--seed",
        "7",
        "--days",
        "1",
        "--seat-model",
        "alice=motivated",
    ]);
    assert!(!ok);
    assert!(
        stderr.contains("'motivated' is not a model this build installs; it installs: rule"),
        "{stderr}"
    );
    let cafe = Cafe::new(
        "decisions-pack-bad",
        "decisions: decisions.yaml\n",
        &[(
            "decisions.yaml",
            "assign:\n  - { seat: otto, model: rule }\n",
        )],
    );
    for command in ["validate", "decisions"] {
        let (ok, _, stderr) = mineworld(&[command, cafe.path()]);
        assert!(!ok, "{command}");
        assert!(
            stderr.contains(":2:13: assign entry 1: 'otto' is a person but not a seat"),
            "{command}: {stderr}"
        );
    }
    let missing = Cafe::new("decisions-pack-missing", "decisions: decisions.yaml\n", &[]);
    let (ok, _, stderr) = mineworld(&["validate", missing.path()]);
    assert!(!ok);
    assert!(
        stderr.contains("decisions: names decisions.yaml, which does not exist"),
        "{stderr}"
    );
}

/// `validate` reports a pack's decision list in one line, and nothing new for a pack without one.
#[test]
fn validate_reports_the_list_only_when_the_pack_names_one() {
    let (ok, stdout, stderr) = mineworld(&["validate", PACK]);
    assert!(ok, "{stderr}");
    assert!(!stdout.contains("decisions"), "{stdout}");
    let cafe = Cafe::new(
        "decisions-validate",
        "decisions: decisions.yaml\n",
        &[(
            "decisions.yaml",
            "assign:\n  - { seat: alice, model: rule }\n  - { seat: bob, model: rule }\n",
        )],
    );
    let (ok, stdout, stderr) = mineworld(&["validate", cafe.path()]);
    assert!(ok, "{stderr}");
    assert!(
        stdout.contains("\n  decisions  2 assignment(s), default rule\n"),
        "{stdout}"
    );
}

/// AC-DLa-11 (QFGb-1): the document the cognition process reads, for a pack whose list assigns by class
/// and by seat and has a memory: section — every seat's model and memory choice, seat > class >
/// default, an unknown policy id passed through, each source the winning entry's position — against a
/// golden written by hand from the fixture's text. The digest, which no hand computes, is the one `run`
/// prints for the same list.
#[test]
fn the_decisions_document_is_the_golden() {
    let classes = "- { class: residents, of: person, tag: resident }\n";
    let memory = std::fs::read_to_string(Path::new(FIXTURES).join("memory.yaml")).expect("reads");
    let cafe = Cafe::new(
        "decisions-json",
        "configure:\n  - classes\ndecisions: decisions.yaml\n",
        &[
            ("configure/classes.yaml", classes),
            ("decisions.yaml", &memory),
        ],
    );
    let (ok, stdout, stderr) = mineworld(&["decisions", cafe.path(), "--json"]);
    assert!(ok, "{stderr}");
    let document: mineworld_decision::ResolvedDecisionsJson =
        serde_json::from_str(&stdout).expect("the typed document reads back");
    let (ok, run, stderr) = mineworld(&[
        "run",
        cafe.path(),
        "--headless",
        "--seed",
        "7",
        "--days",
        "1",
    ]);
    assert!(ok, "{stderr}");
    assert_eq!(
        clause(&run).map(|clause| digest_of(&clause)),
        Some(document.digest.clone())
    );
    let golden = std::fs::read_to_string(Path::new(FIXTURES).join("memory.json"))
        .expect("the golden reads")
        .replace("<digest>", &document.digest);
    assert_eq!(stdout, golden);

    let (ok, stdout, stderr) = mineworld(&["decisions", PACK, "--json"]);
    assert!(ok, "{stderr}");
    let plain: mineworld_decision::ResolvedDecisionsJson =
        serde_json::from_str(&stdout).expect("reads");
    assert_eq!(plain.seats.len(), 11);
    assert!(
        plain
            .seats
            .iter()
            .all(|seat| seat.memory.is_none() && seat.model.id.as_deref() == Some("rule")),
        "no list: every seat rule, every memory null"
    );
}

/// QDLa-1: `server` refuses --decisions and --seat-model without --town, and a seat named by both
/// --agent and --seat-model, before it binds.
#[test]
fn server_refuses_decision_flags_it_could_not_use() {
    let (_file, file) = list_file("decisions-server", "default: rule\n");
    for (arguments, said) in [
        (
            vec![
                "server",
                PACK,
                "--invite",
                INVITE,
                "--decisions",
                file.as_str(),
            ],
            "[mineworld] --decisions: decisions choose how in-server seats decide; add --town",
        ),
        (
            vec![
                "server",
                PACK,
                "--invite",
                INVITE,
                "--seat-model",
                "alice=rule",
            ],
            "[mineworld] --seat-model: decisions choose how in-server seats decide; add --town",
        ),
        (
            vec![
                "server",
                PACK,
                "--invite",
                INVITE,
                "--town",
                "--agent",
                "alice",
                "--seat-model",
                "alice=rule",
            ],
            "seat 'alice' is also an --agent",
        ),
    ] {
        let (ok, stdout, stderr) = mineworld(&arguments);
        assert!(!ok, "{arguments:?}");
        assert!(stderr.contains(said), "{arguments:?}: {stderr}");
        assert!(
            !stdout.contains("listening"),
            "refused before binding: {stdout}"
        );
    }
}

/// SD-DLa-13/-14: `server --town --decisions` starts, drives its seats, and its town: statement ends
/// with the same clause `run` prints for that list.
#[tokio::test]
async fn a_hosted_town_with_a_decision_list_states_its_digest() {
    let (_file, file) = list_file("decisions-town", "default: rule\n");
    let (_ok, run, stderr) = mineworld(&[
        "run",
        PACK,
        "--headless",
        "--seed",
        "0",
        "--days",
        "1",
        "--decisions",
        &file,
    ]);
    let expected = clause(&run).unwrap_or_else(|| panic!("run prints the clause: {stderr}"));
    let (server, output) = Server::start_captured(
        &[
            "server",
            PACK,
            "--town",
            "--invite",
            INVITE,
            "--decisions",
            &file,
        ],
        None,
    )
    .await;
    let deadline = Instant::now() + Duration::from_secs(30);
    let town = loop {
        if let Some(line) = output
            .stdout()
            .into_iter()
            .find(|line| line.contains("driven in-server"))
        {
            break line;
        }
        assert!(
            Instant::now() < deadline,
            "no town line: {:?}",
            output.stdout()
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    assert!(
        // TW-c D-TWc-3: `--pace` is `--cadence` (QTW-13); the digest clause is unchanged.
        town.ends_with(&format!("(town: seed 0, cadence 5 wall s{expected})")),
        "{town}"
    );
    assert!(town.contains("11 seat(s) driven in-server"), "{town}");
    drop(server);
}
