//! Running one page's executed blocks, in order, against its scratch folder (`docs/GUIDE_FORMAT.md` §§4, 5,
//! 6, 8). Every block gets one outcome; a pending block is never reported as passed.

use std::path::Path;
use std::time::{Duration, Instant};

use super::command::{self, Rules};
use super::features::Features;
use super::files;
use super::page::{Block, Body, Directive, Page};
use super::rewrite::{self, Mode};

/// The reason a `cold-start` block is listed as not run in the default mode (§6). Unlike a pending feature,
/// it does not make the blocks after it pending: they never depended on it, because the default run's
/// machine already has what the block installs.
pub const COLD_START: &str = "cold-start";

/// What a page run needs from outside the page: the features the guide is checked against, the
/// repository the committed folders of `equals` are named in (and the one `git clone` reads, §4.6), the
/// reader's home folder the shell blocks see as `HOME` (§4.6), the `mineworld` binary every command runs,
/// the rules (§4.6 or §4.7), and whether a differing console block fails or is re-recorded (§11.2).
pub struct Environment<'a> {
    pub features: &'a Features,
    pub repo: &'a Path,
    pub home: &'a Path,
    pub binary: &'a Path,
    /// Which rules the run follows (§4.6 or §4.7).
    pub rules: Rules,
    /// Whether a console block whose output differs fails, or is re-recorded (§11.2).
    pub mode: Mode,
}

impl<'a> Environment<'a> {
    /// The repository's own guide test (§4.6): the test's own binary, the default rules, check mode.
    pub fn repository(features: &'a Features, repo: &'a Path, home: &'a Path) -> Self {
        Self {
            features,
            repo,
            home,
            binary: Path::new(command::BINARY),
            rules: Rules::Repository,
            mode: Mode::Check,
        }
    }
}

/// What the runner made of one executed block.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The block ran, its output matched, and any `equals` held.
    Passed,
    /// Write mode only (§11.2): the block ran, any `equals` held, and its output differed from the page, so
    /// these are the block's lines as they are to be written: its commands, each followed by what it printed.
    Rewritten(Vec<String>),
    /// The block waits for this feature, or is operator-only (`"operator-only"`). It was not run (§8.1).
    Pending(String),
    /// The block failed; the text is the reason a writer needs to fix it.
    Failed(String),
    /// A shell block for another operating system's shell (§4.6): not run here, never counted as passed,
    /// and run by the same test on that operating system.
    OtherOs(&'static str),
}

/// One executed block's report.
#[derive(Debug, PartialEq, Eq)]
pub struct Report {
    /// The page's name, as the page is listed (its file name without `.md`).
    pub page: String,
    /// The level-3 heading the block sits under, if any.
    pub step: Option<String>,
    /// The line the block begins on.
    pub line: usize,
    /// What happened to the block.
    pub outcome: Outcome,
    /// How long the block took (§11 item 5).
    pub elapsed: Duration,
}

/// Runs `page` in `scratch`. `carried` is the feature a page this one continues is pending on, if any: the
/// scratch folder is then not the state this page expects, so every block is pending too (§8.1, rule 3).
/// Returns one report per executed block, and the pending feature the scratch folder is left with. A
/// `cold-start` block that is not run carries nothing (§6).
pub fn run_page(
    name: &str,
    page: &Page,
    scratch: &Path,
    environment: &Environment,
    carried: Option<String>,
) -> (Vec<Report>, Option<String>) {
    let mut pending = carried;
    let mut reports = Vec::with_capacity(page.blocks.len());
    for block in &page.blocks {
        let began = Instant::now();
        let outcome = match &pending {
            Some(feature) => Outcome::Pending(feature.clone()),
            None => block_outcome(block, scratch, environment),
        };
        if pending.is_none()
            && let Outcome::Pending(feature) = &outcome
            && feature != COLD_START
        {
            pending = Some(feature.clone());
        }
        reports.push(Report {
            page: name.to_owned(),
            step: block.step.clone(),
            line: block.line,
            outcome,
            elapsed: began.elapsed(),
        });
    }
    (reports, pending)
}

fn block_outcome(block: &Block, scratch: &Path, environment: &Environment) -> Outcome {
    let mut packs = None;
    let mut equals = None;
    for directive in &block.directives {
        match directive {
            Directive::Requires(feature) => match environment.features.get(feature) {
                None => {
                    return Outcome::Failed(format!(
                        "requires {feature}: features.toml names no such feature"
                    ));
                }
                Some(false) => return Outcome::Pending(feature.clone()),
                Some(true) => {}
            },
            Directive::OperatorOnly => return Outcome::Pending("operator-only".to_owned()),
            Directive::ColdStart if environment.rules == Rules::Repository => {
                return Outcome::Pending(COLD_START.to_owned());
            }
            Directive::ColdStart => {}
            Directive::Packs(dir) => packs = Some(dir.as_str()),
            Directive::Equals { folder, committed } => equals = Some((folder, committed)),
        }
    }
    let result = match &block.body {
        Body::File { path, text } => files::write(scratch, path, text).map(|()| None),
        Body::Console { exit, lines } => console(
            environment.binary,
            scratch,
            *exit,
            lines,
            packs,
            environment.mode,
        ),
        Body::Shell { family, .. } if !family.is_this_os() => {
            return Outcome::OtherOs(family.word());
        }
        Body::Shell { family, text } => command::shell(
            scratch,
            environment.home,
            environment.repo,
            *family,
            text,
            environment.rules,
        )
        .map(|()| None),
    };
    let rewritten = match result {
        Ok(rewritten) => rewritten,
        Err(reason) => return Outcome::Failed(reason),
    };
    if let Some((folder, committed)) = equals {
        let checked = files::confine(folder).and_then(|folder| {
            files::same_tree(&scratch.join(folder), &environment.repo.join(committed))
        });
        if let Err(reason) = checked {
            return Outcome::Failed(format!("equals {folder} {committed}: {reason}"));
        }
    }
    rewritten.map_or(Outcome::Passed, Outcome::Rewritten)
}

/// Runs each command of a console block and matches what it printed against the lines under it (§4). In
/// write mode a difference is not a failure: the block's lines with each differing command's output recorded
/// (§11.2) are returned instead. An exit code other than the block's is a failure in both modes.
fn console(
    binary: &Path,
    scratch: &Path,
    exit: i32,
    lines: &[String],
    packs: Option<&str>,
    mode: Mode,
) -> Result<Option<Vec<String>>, String> {
    let mut written = Vec::new();
    let mut changed = false;
    for (command_line, expected) in commands(lines) {
        let mut words = command::words(&command_line)?;
        if let Some(dir) = packs {
            words.push("--packs".to_owned());
            words.push(dir.to_owned());
        }
        let (actual, code) = command::run(binary, scratch, &words)?;
        if code != exit {
            return Err(format!(
                "`{command_line}` exited with {code}, the page expects {exit}:\n{actual}"
            ));
        }
        written.push(format!("$ {command_line}"));
        match (command::matches(&expected, &actual), mode) {
            (Ok(()), _) => written.extend(expected.lines().map(str::to_owned)),
            (Err(difference), Mode::Check) => {
                return Err(format!(
                    "`{command_line}` printed other output than the page shows:\n{difference}"
                ));
            }
            (Err(_), Mode::Write) => {
                changed = true;
                written.extend(
                    rewrite::recorded(&expected, &actual)
                        .lines()
                        .map(str::to_owned),
                );
            }
        }
    }
    Ok(changed.then_some(written))
}

/// A console block as (command, expected output) pairs. The block's first line is a command, which parsing
/// guarantees, so every output line has a command above it.
pub fn commands(lines: &[String]) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    for line in lines {
        if let Some(command_line) = line.strip_prefix("$ ") {
            pairs.push((command_line.to_owned(), String::new()));
        } else if let Some((_, expected)) = pairs.last_mut() {
            expected.push_str(line);
            expected.push('\n');
        }
    }
    pairs
}
