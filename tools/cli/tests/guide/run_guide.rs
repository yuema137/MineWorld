//! Running a whole guide: its pages in order, each in its scratch folder, then the guards that hold the pages
//! to their mirrors, their links, and the features they name (`docs/GUIDE_FORMAT.md` §§1, 5, 7–9, 11).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use mineworld_test_support::Scratch;

use super::command;
use super::features::{self, Features};
use super::mirror;
use super::page;
use super::page_run::{Environment, Outcome, Report, run_page};
use super::report::{Summary, Verdict};

/// Runs the guide at `guide` (a folder holding `features.toml` and the pages) against the repository `repo`,
/// which holds the committed folders `equals` names and the generated `docs/reference`. `label` names the
/// scratch folders, so two runs in one process never share one.
pub fn run(label: &str, guide: &Path, repo: &Path) -> Summary {
    let mut summary = Summary::default();
    let features = match read_features(guide) {
        Ok(features) => features,
        Err(reason) => {
            summary
                .checks
                .push(("features.toml".to_owned(), Verdict::Fail(reason)));
            return summary;
        }
    };
    let pages = match english_pages(guide) {
        Ok(pages) => pages,
        Err(reason) => {
            summary
                .checks
                .push(("the guide folder".to_owned(), Verdict::Fail(reason)));
            return summary;
        }
    };
    run_pages(label, &pages, &features, repo, &mut summary);
    guard_pages(guide, &pages, repo, &mut summary);
    check_features(guide, &features, &mut summary);
    summary
}

fn read_features(guide: &Path) -> Result<Features, String> {
    let text = fs::read_to_string(guide.join("features.toml"))
        .map_err(|error| format!("features.toml cannot be read: {error}"))?;
    features::read(&text)
}

/// Every English page, in file-name order. A page is a Markdown file directly in the guide folder, other than
/// the index (`README.md` and its mirror `README.zh-CN.md`) and mirrors. A mirror without its English page is
/// a failure (§1.2).
fn english_pages(guide: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(guide)
        .map_err(|error| format!("{} cannot be read: {error}", guide.display()))?;
    let mut english = Vec::new();
    let mut mirrors = Vec::new();
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !path.is_file() || !name.ends_with(".md") || INDEX.contains(&name) {
            continue;
        }
        if let Some(stem) = name.strip_suffix(".zh-CN.md") {
            mirrors.push(stem.to_owned());
        } else {
            english.push(path);
        }
    }
    english.sort();
    for mirror in &mirrors {
        if !english.iter().any(|page| page_name(page) == mirror) {
            return Err(format!("{mirror}.zh-CN.md has no English page {mirror}.md"));
        }
    }
    Ok(english)
}

/// The guide's index and its mirror: read by people, never executed, and not pages (§1).
const INDEX: [&str; 2] = ["README.md", "README.zh-CN.md"];

fn page_name(path: &Path) -> &str {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
}

/// Runs every page's executed blocks in order. A page that continues an earlier one uses its scratch folder
/// and inherits its pending state (§5, §8.1).
fn run_pages(
    label: &str,
    pages: &[PathBuf],
    features: &Features,
    repo: &Path,
    summary: &mut Summary,
) {
    let environment = Environment { features, repo };
    let mut folders: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut carries: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut kept: Vec<Scratch> = Vec::new();
    for path in pages {
        let name = page_name(path).to_owned();
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                summary
                    .checks
                    .push((name, Verdict::Fail(error.to_string())));
                continue;
            }
        };
        let parsed = match page::parse(&source) {
            Ok(parsed) => parsed,
            Err(reason) => {
                summary
                    .checks
                    .push((format!("page {name}"), Verdict::Fail(reason)));
                continue;
            }
        };
        let (folder, carried) = match &parsed.continues {
            Some(previous) => match (folders.get(previous), carries.get(previous)) {
                (Some(folder), Some(carried)) => (folder.clone(), carried.clone()),
                _ => {
                    summary.checks.push((
                        format!("page {name}"),
                        Verdict::Fail(format!(
                            "continues {previous}: no earlier page of that name ran"
                        )),
                    ));
                    continue;
                }
            },
            None => {
                let scratch = mineworld_test_support::scratch!(empty format!("{label}-{name}"));
                let folder = scratch.path().to_path_buf();
                kept.push(scratch);
                (folder, None)
            }
        };
        let (reports, carry) = run_page(&name, &parsed, &folder, &environment, carried);
        folders.insert(name.clone(), folder);
        carries.insert(name, carry);
        record(reports, summary);
    }
}

/// Counts each block, lists pending blocks by page, step and feature, and records each failure by name.
fn record(reports: Vec<Report>, summary: &mut Summary) {
    for report in reports {
        let step = report.step.as_deref().unwrap_or("(no step)");
        let place = format!("page {} line {} step {step}", report.page, report.line);
        match report.outcome {
            Outcome::Passed => summary.passed += 1,
            Outcome::Pending(feature) => {
                summary
                    .pending
                    .push(format!("{place}: waits for {feature}"));
            }
            Outcome::Failed(reason) => summary.checks.push((place, Verdict::Fail(reason))),
        }
    }
}

/// §1.2, §7 and §8.2–8.3: each English page with its mirror, and its links.
fn guard_pages(guide: &Path, pages: &[PathBuf], repo: &Path, summary: &mut Summary) {
    let reference = mirror::normalise(&repo.join("docs/reference"));
    let reference_present = reference.is_dir();
    if !reference_present {
        summary.checks.push((
            "reference links".to_owned(),
            Verdict::Inconclusive(
                "docs/reference is generated by PR 25a, which has not merged; links into it are not checked"
                    .to_owned(),
            ),
        ));
    }
    for path in pages {
        let name = page_name(path);
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                summary
                    .checks
                    .push((format!("page {name}"), Verdict::Fail(error.to_string())));
                continue;
            }
        };
        let english = mirror::outline(&source);
        let mirror_path = guide.join(format!("{name}.zh-CN.md"));
        let mut problems = mirror::link_problems(
            &mirror::normalise(path),
            &english,
            &reference,
            reference_present,
        );
        let mirror_verdict = match fs::read_to_string(&mirror_path) {
            Err(_) => Verdict::Fail(format!("{name}.md has no mirror {name}.zh-CN.md")),
            Ok(mirror_source) => {
                let translated = mirror::outline(&mirror_source);
                // §7 rule 4: the mirror's own links resolve too, by the same rule as the page's.
                problems.extend(
                    mirror::link_problems(
                        &mirror::normalise(&mirror_path),
                        &translated,
                        &reference,
                        reference_present,
                    )
                    .into_iter()
                    .map(|problem| format!("{name}.zh-CN.md: {problem}")),
                );
                match mirror::mirror_problem(&english, &translated) {
                    None => Verdict::Pass,
                    Some(problem) => Verdict::Fail(problem),
                }
            }
        };
        summary
            .checks
            .push((format!("mirror of {name}"), mirror_verdict));
        let links = if problems.is_empty() {
            Verdict::Pass
        } else {
            Verdict::Fail(problems.join("\n"))
        };
        summary.checks.push((format!("links of {name}"), links));
    }
}

/// §9 rule 3: the flags against the packs this build lists, asked of the binary itself.
fn check_features(guide: &Path, features: &Features, summary: &mut Summary) {
    let words = ["mineworld", "packs", "list"].map(str::to_owned);
    match command::run(guide, &words) {
        Ok((listing, 0)) => {
            for (feature, verdict) in features::against_build(features, &listing) {
                summary.checks.push((format!("feature {feature}"), verdict));
            }
        }
        Ok((listing, code)) => summary.checks.push((
            "packs list".to_owned(),
            Verdict::Fail(format!("exited with {code}:\n{listing}")),
        )),
        Err(reason) => summary
            .checks
            .push(("packs list".to_owned(), Verdict::Fail(reason))),
    }
}
