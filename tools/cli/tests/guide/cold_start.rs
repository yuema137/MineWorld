//! The cold-start run (`docs/GUIDE_FORMAT.md` §4.7, `ARC-362`): the guide followed on a clean runner, from
//! source (layer A) or from the downloaded bundle (layer B). `.github/workflows/cold-start.yml` writes a
//! configuration file through `scripts/ci_cold_start.py`, names it in `MINEWORLD_COLD_START`, and runs the
//! ignored test `cold_start`, once for the guide and once for the substitutes of the operator-only blocks.
//!
//! The same runner and the same rules as the repository's own guide test; only `RunOptions` differ. A run
//! that proved nothing (the clone or the bundle is not the commit this test was built from) is INCONCLUSIVE,
//! which fails the test: it is never reported as a pass.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

use super::command::{self, Mode};
use super::files;
use super::page::Family;
use super::report::{Summary, Verdict};
use super::run_guide::{self, Mutation, RunOptions};
use super::substitutes;

/// The environment variable that names the configuration file.
pub const VARIABLE: &str = "MINEWORLD_COLD_START";

/// The page whose shell blocks clone and build MineWorld (§4.7 rule 5).
pub const BUILD_PAGE: &str = "01-get-mineworld";

/// Pages whose scratch folder the substitutes need as it stood when the page ended: 06.2 serves the world
/// chapter 06 left, and 14.2 the one chapter 14 left (later pages continue the same folder and change it).
pub const SNAPSHOT_PAGES: [&str; 2] = ["06-play-it", "14-play-as-an-agent"];

/// The committed expectation of each layer's run (§4.7 rule 10), built into the test executable, because the
/// clean runner has no checkout to read it from.
const EXPECTED: &str = include_str!("expected_cold_start.toml");

/// The committed page mutations (§4.7 rule 12), built in for the same reason.
const MUTATIONS: &str = include_str!("cold_start_mutations.toml");

/// Which question the run answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    /// From source: install, clone, build, then the guide with the reader's binary.
    A,
    /// From the download: the guide's chapters 02–06 with the bundle's binary.
    B,
}

impl Layer {
    fn key(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
        }
    }
}

/// Which half of the run this invocation is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// The guide's pages (§4.7 rules 1–10).
    Guide,
    /// The substitutes for operator-only blocks (§4.7 rule 11), after the guide phase.
    Substitutes,
}

/// What `scripts/ci_cold_start.py config` wrote.
#[derive(Clone, Debug)]
pub struct Config {
    pub layer: Layer,
    pub phase: Phase,
    /// The pages this run reads: the harness's copy of `docs/guide`.
    pub guide: PathBuf,
    /// The repository `equals` and the guards read: A's clone, or B's harness folder.
    pub repo: PathBuf,
    /// A: where chapter 01 clones MineWorld.
    pub clone: Option<PathBuf>,
    /// The `mineworld` every command runs.
    pub binary: PathBuf,
    /// B: the unpacked bundle's folder.
    pub bundle: Option<PathBuf>,
    /// The pages to run; empty runs all.
    pub pages: Vec<String>,
    /// The commit this test executable was built from.
    pub expected_commit: String,
    /// Whether this run is pre-merge evidence (a branch other than `main`, §4.7 rule 8).
    pub pre_merge: bool,
    /// Where scratch folders, snapshots and scripts go.
    pub scratch_root: PathBuf,
    /// The folder the report files are written to.
    pub report: PathBuf,
    /// A committed page mutation's name, or empty.
    pub mutation: String,
    /// Substitutes: the pinned Godot, when the 2D client's drive is to run (Linux and macOS).
    pub godot: Option<PathBuf>,
}

/// Reads the configuration the variable names. A missing variable or file is an error that names the
/// workflow: the test never passes by doing nothing (§4.7).
pub fn load(variable: Option<OsString>) -> Result<Config, String> {
    let path = variable.filter(|value| !value.is_empty()).ok_or_else(|| {
        format!(
            "cold_start is run by .github/workflows/cold-start.yml with {VARIABLE} set; see \
             docs/GUIDE_FORMAT.md §4.7"
        )
    })?;
    let path = PathBuf::from(path);
    let text = std::fs::read_to_string(&path).map_err(|error| {
        format!(
            "{VARIABLE} names {}, which cannot be read ({error}); see docs/GUIDE_FORMAT.md §4.7",
            path.display()
        )
    })?;
    parse(&text)
}

/// Parses a configuration file's text.
pub fn parse(text: &str) -> Result<Config, String> {
    let table: toml::Table = text
        .parse()
        .map_err(|error| format!("the cold-start configuration is not TOML: {error}"))?;
    let text_of = |key: &str| -> Result<String, String> {
        table
            .get(key)
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("the cold-start configuration has no text {key:?}"))
    };
    let optional = |key: &str| {
        table
            .get(key)
            .and_then(toml::Value::as_str)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };
    let layer = match text_of("layer")?.as_str() {
        "a" => Layer::A,
        "b" => Layer::B,
        other => return Err(format!("layer {other:?} is neither \"a\" nor \"b\"")),
    };
    let phase = match text_of("phase")?.as_str() {
        "guide" => Phase::Guide,
        "substitutes" => Phase::Substitutes,
        other => return Err(format!("phase {other:?} is neither \"guide\" nor \"substitutes\"")),
    };
    let pages = table
        .get("pages")
        .and_then(toml::Value::as_array)
        .ok_or("the cold-start configuration has no list \"pages\"")?
        .iter()
        .map(|page| page.as_str().map(str::to_owned).ok_or("a page is text"))
        .collect::<Result<Vec<_>, _>>()?;
    let expected_commit = text_of("expected_commit")?;
    if expected_commit.len() != 40 || !expected_commit.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("expected_commit {expected_commit:?} is not a commit id"));
    }
    let config = Config {
        layer,
        phase,
        guide: PathBuf::from(text_of("guide")?),
        repo: PathBuf::from(text_of("repo")?),
        clone: optional("clone"),
        binary: PathBuf::from(text_of("binary")?),
        bundle: optional("bundle"),
        pages,
        expected_commit,
        pre_merge: table
            .get("pre_merge")
            .and_then(toml::Value::as_bool)
            .ok_or("the cold-start configuration has no boolean \"pre_merge\"")?,
        scratch_root: PathBuf::from(text_of("scratch_root")?),
        report: PathBuf::from(text_of("report")?),
        mutation: text_of("mutation")?,
        godot: optional("godot"),
    };
    match (config.layer, &config.clone, &config.bundle) {
        (Layer::A, Some(_), None) | (Layer::B, None, Some(_)) => Ok(config),
        _ => Err("layer a names a clone and no bundle; layer b a bundle and no clone".to_owned()),
    }
}

/// A committed page mutation by name (§4.7 rule 12). An unknown name is an error.
pub fn mutation(name: &str) -> Result<Mutation, String> {
    let table: toml::Table = MUTATIONS
        .parse()
        .map_err(|error| format!("cold_start_mutations.toml is not TOML: {error}"))?;
    let entry = table
        .get(name)
        .and_then(toml::Value::as_table)
        .ok_or_else(|| {
            let known: Vec<&str> = table.keys().map(String::as_str).collect();
            format!("no committed mutation {name:?}; the committed ones are {known:?}")
        })?;
    let field = |key: &str| {
        entry
            .get(key)
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("mutation {name:?} has no text {key:?}"))
    };
    Ok(Mutation {
        name: name.to_owned(),
        page: field("page")?,
        old: field("old")?,
        new: field("new")?,
    })
}

/// A layer's committed expectation: the blocks it does not run, and the fewest that must pass.
pub fn expected(layer: Layer) -> Result<(Vec<String>, usize), String> {
    let table: toml::Table = EXPECTED
        .parse()
        .map_err(|error| format!("expected_cold_start.toml is not TOML: {error}"))?;
    let entry = table
        .get(layer.key())
        .and_then(toml::Value::as_table)
        .ok_or_else(|| format!("expected_cold_start.toml has no [{}]", layer.key()))?;
    let minimum = entry
        .get("minimum_passed")
        .and_then(toml::Value::as_integer)
        .and_then(|count| usize::try_from(count).ok())
        .ok_or("minimum_passed is a count")?;
    let mut not_run = entry
        .get("not_run")
        .and_then(toml::Value::as_array)
        .ok_or("not_run is a list")?
        .iter()
        .map(|item| item.as_str().map(str::to_owned).ok_or("each entry is text"))
        .collect::<Result<Vec<_>, _>>()?;
    not_run.sort();
    Ok((not_run, minimum))
}

/// The real home of the runner's user (§4.7 rule 1).
fn real_home() -> Result<PathBuf, String> {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .map(PathBuf::from)
        .ok_or_else(|| "the process has no home folder".to_owned())
}

/// The whole cold-start invocation: the configured phase, its report written, and an error when the run did
/// not pass (a failure, or an INCONCLUSIVE gate check).
pub fn run(config: &Config) -> Result<(), String> {
    std::fs::create_dir_all(&config.report)
        .map_err(|error| format!("{} cannot be made: {error}", config.report.display()))?;
    match config.phase {
        Phase::Guide => guide_phase(config),
        Phase::Substitutes => substitutes::run(config),
    }
}

/// A gate check's name: the checks this module adds, which an INCONCLUSIVE verdict fails (§4.7 rule 8).
fn gate(name: &str) -> String {
    format!("cold-start: {name}")
}

fn guide_phase(config: &Config) -> Result<(), String> {
    let home = real_home()?;
    let mutation = match config.mutation.as_str() {
        "" => None,
        name => Some(mutation(name)?),
    };
    let scratch_root = config.scratch_root.to_string_lossy().into_owned();
    let options = RunOptions {
        mode: Mode::ColdStart,
        binary: &config.binary,
        home: Some(&home),
        pages: &config.pages,
        scratch_root: &scratch_root,
        guards: config.layer == Layer::A,
        mutation: mutation.as_ref(),
    };
    let mut before = Vec::new();
    if config.layer == Layer::B {
        before.push(bundle_check(config));
    }
    let mut summary = if before.iter().any(|(_, verdict)| *verdict != Verdict::Pass) {
        Summary::default()
    } else {
        let mut after = |name: &str, folder: &Path| after_page(config, &home, name, folder);
        run_guide::run_with(
            &format!("cold-{}", config.layer.key()),
            &config.guide,
            &config.repo,
            &options,
            &mut after,
        )
    };
    summary.checks.splice(0..0, before);
    held_to_expectation(config.layer, &mut summary);
    let verdict = verdict_of(&summary);
    write_report(config, &summary, verdict)?;
    let _ = std::io::Write::write_all(
        &mut std::io::stderr(),
        format!("{}\nverdict: {verdict}\n", summary.render()).as_bytes(),
    );
    match verdict {
        "PASS" => Ok(()),
        _ => Err(format!(
            "the cold-start run is {verdict}:\n{}",
            summary.render()
        )),
    }
}

/// PASS, FAIL, or INCONCLUSIVE when nothing failed but a gate check could not prove its point.
fn verdict_of(summary: &Summary) -> &'static str {
    if !summary.no_failures() {
        "FAIL"
    } else if summary.checks.iter().any(|(name, verdict)| {
        name.starts_with("cold-start: ") && matches!(verdict, Verdict::Inconclusive(_))
    }) {
        "INCONCLUSIVE"
    } else {
        "PASS"
    }
}

/// §4.7 rule 10: the blocks not run are exactly the committed list, and at least the committed number passed.
fn held_to_expectation(layer: Layer, summary: &mut Summary) {
    let verdict = match expected(layer) {
        Err(reason) => Verdict::Fail(reason),
        Ok((not_run, minimum)) => {
            let mut actual = summary.not_run.clone();
            actual.sort();
            if actual != not_run {
                Verdict::Fail(format!(
                    "the blocks that did not run are not the committed list \
                     (tools/cli/tests/guide/expected_cold_start.toml [{}]):\n  ran without: {actual:#?}\n  \
                     committed: {not_run:#?}",
                    layer.key()
                ))
            } else if summary.passed < minimum {
                Verdict::Fail(format!(
                    "{} block(s) passed, fewer than the committed minimum of {minimum}",
                    summary.passed
                ))
            } else {
                Verdict::Pass
            }
        }
    };
    summary.checks.push((gate("expectation"), verdict));
}

/// What the run checks after a page: after chapter 01 in layer A, the clone, its commit, its pages and the
/// reader's binary; after the pages the substitutes need, a copy of their folder.
fn after_page(config: &Config, home: &Path, name: &str, folder: &Path) -> Vec<(String, Verdict)> {
    let mut checks = Vec::new();
    if config.layer == Layer::A && name == BUILD_PAGE {
        checks.extend(after_build(config, home));
    }
    if SNAPSHOT_PAGES.contains(&name) {
        let copy = config.scratch_root.join(format!("after-{name}"));
        let _ = std::fs::remove_dir_all(&copy);
        let verdict = match copy_tree(folder, &copy) {
            Ok(()) => Verdict::Pass,
            Err(reason) => Verdict::Fail(reason),
        };
        checks.push((gate(&format!("snapshot after {name}")), verdict));
    }
    checks
}

/// §4.7 rules 5 and 8, after page 01 of layer A.
fn after_build(config: &Config, home: &Path) -> Vec<(String, Verdict)> {
    let clone = config.clone.as_deref().expect("layer a names its clone");
    let mut checks = Vec::new();
    if !clone.join(".git").exists() {
        checks.push((
            gate("clone"),
            Verdict::Fail(format!("chapter 01 made no clone at {}", clone.display())),
        ));
        return checks;
    }
    if config.pre_merge {
        let verdict = match pre_merge_checkout(config, clone, home) {
            Ok(()) => Verdict::Pass,
            Err(reason) => Verdict::Fail(reason),
        };
        checks.push((
            gate(&format!(
                "PRE-MERGE: clone moved to {} and built again",
                config.expected_commit
            )),
            verdict,
        ));
    }
    let head = git_head(clone);
    checks.push((
        gate("the clone is the harness's commit"),
        match head {
            Ok(head) if head == config.expected_commit => Verdict::Pass,
            Ok(head) => Verdict::Inconclusive(format!(
                "the clone is at {head}, the harness was built from {}: main moved between the harness \
                 build and the clone; re-run",
                config.expected_commit
            )),
            Err(reason) => Verdict::Fail(reason),
        },
    ));
    checks.push((
        gate("the clone's docs/guide is the pages this run read"),
        match files::same_tree(&clone.join("docs/guide"), &config.guide) {
            Ok(()) => Verdict::Pass,
            Err(reason) => Verdict::Inconclusive(format!(
                "the clone's docs/guide differs from the harness's: {reason}"
            )),
        },
    ));
    let named = clone
        .join("target")
        .join("release")
        .join(format!("mineworld{}", std::env::consts::EXE_SUFFIX));
    checks.push((
        gate("the reader's binary"),
        if !config.binary.is_file() {
            Verdict::Fail(format!(
                "the reader's build made no binary at {}",
                config.binary.display()
            ))
        } else if config.binary != named {
            Verdict::Fail(format!(
                "the run uses {}, which is not the file chapter 01 names ({})",
                config.binary.display(),
                named.display()
            ))
        } else {
            Verdict::Pass
        },
    ));
    checks
}

/// Pre-merge only (§4.7 rule 8): the clone read `main`; check the branch's commit out and build it again,
/// as a new terminal would, so the rest of the run reads and runs the commit under test.
fn pre_merge_checkout(config: &Config, clone: &Path, home: &Path) -> Result<(), String> {
    let sha = &config.expected_commit;
    let quoted = clone.to_string_lossy();
    let (family, text) = if cfg!(windows) {
        (
            Family::PowerShell,
            format!(
                "cd \"{quoted}\"\ngit fetch --quiet origin {sha}\ngit checkout --quiet --detach {sha}\n\
                 cargo build --release -p mineworld-cli\n"
            ),
        )
    } else {
        (
            Family::Posix,
            format!(
                "cd \"{quoted}\"\ngit fetch --quiet origin {sha}\ngit checkout --quiet --detach {sha}\n\
                 cargo build --release -p mineworld-cli\n"
            ),
        )
    };
    let scripts = config.scratch_root.join("pre-merge");
    std::fs::create_dir_all(&scripts).map_err(|error| error.to_string())?;
    command::shell(&scripts, home, clone, family, &text, Mode::ColdStart)
}

fn git_head(clone: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(clone)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("git cannot be started: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git rev-parse HEAD in the clone failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Layer B, before any page (§4.7 rules 5 and 8): the binary is the bundle's server, and the bundle is the
/// commit this test was built from.
fn bundle_check(config: &Config) -> (String, Verdict) {
    let bundle = config.bundle.as_deref().expect("layer b names its bundle");
    let runtime = bundle.join("runtime");
    let text = match std::fs::read_to_string(runtime.join("BUNDLE.toml")) {
        Ok(text) => text,
        Err(error) => {
            return (
                gate("bundle"),
                Verdict::Fail(format!("{}/BUNDLE.toml cannot be read: {error}", runtime.display())),
            );
        }
    };
    let table: toml::Table = match text.parse() {
        Ok(table) => table,
        Err(error) => return (gate("bundle"), Verdict::Fail(format!("BUNDLE.toml: {error}"))),
    };
    let field = |key: &str| table.get(key).and_then(toml::Value::as_str).unwrap_or("");
    let server = runtime.join(field("server"));
    if field("server").is_empty() || server != config.binary || !server.is_file() {
        return (
            gate("bundle"),
            Verdict::Fail(format!(
                "BUNDLE.toml names the server {:?}; the run uses {}, which must be that file in {}",
                field("server"),
                config.binary.display(),
                runtime.display()
            )),
        );
    }
    let commit = field("commit");
    if commit == config.expected_commit {
        (gate("the bundle is the harness's commit"), Verdict::Pass)
    } else if config.pre_merge {
        (
            gate(&format!(
                "PRE-MERGE: bundle from {commit}, harness from {}; not acceptance evidence",
                config.expected_commit
            )),
            Verdict::Pass,
        )
    } else {
        (
            gate("the bundle is the harness's commit"),
            Verdict::Inconclusive(format!(
                "the bundle was built from {commit:?}, the harness from {}; re-run with a bundle of this commit",
                config.expected_commit
            )),
        )
    }
}

/// Copies the folder `from` to `to`, files and folders only.
pub fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|error| format!("{} cannot be made: {error}", to.display()))?;
    let entries =
        std::fs::read_dir(from).map_err(|error| format!("{} cannot be read: {error}", from.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if kind.is_file() {
            std::fs::copy(entry.path(), &target)
                .map_err(|error| format!("{} cannot be copied: {error}", entry.path().display()))?;
        }
    }
    Ok(())
}

/// `report/guide.json`: what `ci_cold_start.py summary` renders (§4.7, §11 item 5).
fn write_report(config: &Config, summary: &Summary, verdict: &str) -> Result<(), String> {
    let failed_pages: Vec<&str> = summary
        .block_times
        .iter()
        .filter(|block| block.outcome == "FAIL")
        .map(|block| block.page.as_str())
        .collect();
    let pages: Vec<Value> = summary
        .timings
        .iter()
        .map(|(name, time)| {
            json!({
                "name": name,
                "seconds": time.as_secs_f64(),
                "verdict": if failed_pages.contains(&name.as_str()) { "FAIL" } else { "PASS" },
            })
        })
        .collect();
    let blocks: Vec<Value> = summary
        .block_times
        .iter()
        .map(|block| {
            json!({
                "page": block.page,
                "step": block.step,
                "line": block.line,
                "seconds": block.elapsed.as_secs_f64(),
                "outcome": block.outcome,
            })
        })
        .collect();
    let checks: Vec<Value> = summary
        .checks
        .iter()
        .filter(|(_, verdict)| *verdict != Verdict::Pass)
        .map(|(name, verdict)| json!({ "name": name, "verdict": format!("{verdict:?}") }))
        .collect();
    let gates: Vec<Value> = summary
        .checks
        .iter()
        .filter(|(name, _)| name.starts_with("cold-start: "))
        .map(|(name, verdict)| {
            json!({ "name": name, "verdict": match verdict {
                Verdict::Pass => "PASS",
                Verdict::Fail(_) => "FAIL",
                Verdict::Inconclusive(_) => "INCONCLUSIVE",
            }})
        })
        .collect();
    let report = json!({
        "layer": config.layer.key(),
        "os": std::env::consts::OS,
        "commit": config.expected_commit,
        "pre_merge": config.pre_merge,
        "binary": config.binary.to_string_lossy(),
        "mutation": config.mutation,
        "verdict": verdict,
        "passed": summary.passed,
        "blocks_total": summary.blocks,
        "not_run": summary.not_run,
        "pages": pages,
        "blocks": blocks,
        "gates": gates,
        "problems": checks,
        "render": summary.render(),
    });
    let path = config.report.join("guide.json");
    std::fs::write(&path, serde_json::to_string_pretty(&report).unwrap_or_default())
        .map_err(|error| format!("{} cannot be written: {error}", path.display()))
}
