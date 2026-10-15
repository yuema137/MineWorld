//! Running one page's executed blocks, in order, against its scratch folder (`docs/GUIDE_FORMAT.md` §§4, 5,
//! 6, 8). Every block gets one outcome; a pending block is never reported as passed.

use std::path::Path;

use super::command;
use super::features::Features;
use super::files;
use super::page::{Block, Body, Directive, Page};

/// What the runner made of one executed block.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The block ran and its output matched.
    Passed,
    /// The block waits for this feature, or is operator-only (`"operator-only"`). It was not run (§8.1).
    Pending(String),
    /// The block failed; the text is the reason a writer needs to fix it.
    Failed(String),
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
}

/// Runs `page` in `scratch`. `carried` is the feature a page this one continues is pending on, if any: the
/// scratch folder is then not the state this page expects, so every block is pending too (§8.1, rule 3).
/// Returns one report per executed block, and the pending feature the scratch folder is left with.
pub fn run_page(
    name: &str,
    page: &Page,
    scratch: &Path,
    features: &Features,
    carried: Option<String>,
) -> (Vec<Report>, Option<String>) {
    let mut pending = carried;
    let mut reports = Vec::with_capacity(page.blocks.len());
    for block in &page.blocks {
        let outcome = match &pending {
            Some(feature) => Outcome::Pending(feature.clone()),
            None => block_outcome(block, scratch, features),
        };
        if pending.is_none()
            && let Outcome::Pending(feature) = &outcome
        {
            pending = Some(feature.clone());
        }
        reports.push(Report {
            page: name.to_owned(),
            step: block.step.clone(),
            line: block.line,
            outcome,
        });
    }
    (reports, pending)
}

fn block_outcome(block: &Block, scratch: &Path, features: &Features) -> Outcome {
    let mut packs = None;
    for directive in &block.directives {
        match directive {
            Directive::Requires(feature) => match features.get(feature) {
                None => {
                    return Outcome::Failed(format!(
                        "requires {feature}: features.toml names no such feature"
                    ));
                }
                Some(false) => return Outcome::Pending(feature.clone()),
                Some(true) => {}
            },
            Directive::OperatorOnly => return Outcome::Pending("operator-only".to_owned()),
            Directive::Packs(dir) => packs = Some(dir.as_str()),
            Directive::Equals(_) => {
                return Outcome::Failed(
                    "equals is not checked by this build of the runner".to_owned(),
                );
            }
        }
    }
    let result = match &block.body {
        Body::File { path, text } => files::write(scratch, path, text),
        Body::Copy { from, to } => files::copy_tree(scratch, from, to),
        Body::Console { exit, lines } => console(scratch, *exit, lines, packs),
    };
    match result {
        Ok(()) => Outcome::Passed,
        Err(reason) => Outcome::Failed(reason),
    }
}

/// Runs each command of a console block and matches what it printed against the lines under it (§4).
fn console(scratch: &Path, exit: i32, lines: &[String], packs: Option<&str>) -> Result<(), String> {
    for (command_line, expected) in commands(lines) {
        let mut words = command::words(&command_line)?;
        if let Some(dir) = packs {
            words.push("--packs".to_owned());
            words.push(dir.to_owned());
        }
        let (actual, code) = command::run(scratch, &words)?;
        if code != exit {
            return Err(format!(
                "`{command_line}` exited with {code}, the page expects {exit}:\n{actual}"
            ));
        }
        command::matches(&expected, &actual).map_err(|difference| {
            format!("`{command_line}` printed other output than the page shows:\n{difference}")
        })?;
    }
    Ok(())
}

/// A console block as (command, expected output) pairs. The block's first line is a command, which parsing
/// guarantees, so every output line has a command above it.
fn commands(lines: &[String]) -> Vec<(String, String)> {
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
