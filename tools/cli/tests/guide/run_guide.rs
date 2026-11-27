//! Running a whole guide: its pages in order, each in its scratch folder, then the guards that hold the pages
//! to their mirrors, their links, and the features they name (`docs/GUIDE_FORMAT.md` §§1, 5, 7–9, 11). The
//! same run serves the repository's own test (§4.6) and the cold-start run (§4.7); `RunOptions` is the
//! whole difference.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use mineworld_test_support::Scratch;

use super::command::{self, Mode};
use super::features::{self, Build, Features};
use super::mirror;
use super::page;
use super::page_run::{Environment, Outcome, Report, run_page};
use super::report::{BlockTime, Summary, Verdict};

/// A committed page mutation (§4.7 rule 12): the exact text `old` of page `page` becomes `new`, in memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mutation {
    pub name: String,
    pub page: String,
    pub old: String,
    pub new: String,
}

/// How a run differs from the repository's own guide test (§4.7).
pub struct RunOptions<'a> {
    pub mode: Mode,
    /// The `mineworld` every console command and feature probe runs.
    pub binary: &'a Path,
    /// The reader's home; `None` makes a scratch home for the run (§4.6 rule 3).
    pub home: Option<&'a Path>,
    /// The pages to run, by name; empty runs every page (§4.7 rule 7).
    pub pages: &'a [String],
    /// The folder scratch folders are made under.
    pub scratch_root: &'a str,
    /// Whether mirror parity and links are checked (§4.7 rule 9).
    pub guards: bool,
    /// A page mutation, applied in memory (§4.7 rule 12).
    pub mutation: Option<&'a Mutation>,
}

impl RunOptions<'static> {
    /// The repository's own guide test (§4.6).
    pub fn repository() -> Self {
        Self {
            mode: Mode::Repository,
            binary: Path::new(command::BINARY),
            home: None,
            pages: &[],
            scratch_root: env!("CARGO_TARGET_TMPDIR"),
            guards: true,
            mutation: None,
        }
    }
}

/// What a run asks after each page, given the page's name and its scratch folder: the checks to record. A
/// check that does not pass stops the run there (§4.7 rules 5 and 8).
pub type AfterPage<'a> = dyn FnMut(&str, &Path) -> Vec<(String, Verdict)> + 'a;

/// Runs the guide at `guide` (a folder holding `features.toml` and the pages) against the repository `repo`,
/// which holds the committed folders `equals` names and the generated `docs/reference`. `label` names the
/// scratch folders, so two runs in one process never share one.
pub fn run(label: &str, guide: &Path, repo: &Path) -> Summary {
    run_with(
        label,
        guide,
        repo,
        &RunOptions::repository(),
        &mut |_, _| Vec::new(),
    )
}

/// [`run`] with the options of a mode, and a check after each page.
pub fn run_with(
    label: &str,
    guide: &Path,
    repo: &Path,
    options: &RunOptions,
    after_page: &mut AfterPage,
) -> Summary {
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
    let pages = match english_pages(guide).and_then(|pages| selected(pages, options.pages)) {
        Ok(pages) => pages,
        Err(reason) => {
            summary
                .checks
                .push(("the guide folder".to_owned(), Verdict::Fail(reason)));
            return summary;
        }
    };
    let stopped = run_pages(label, &pages, &features, repo, options, after_page, &mut summary);
    if stopped {
        return summary;
    }
    if options.guards {
        guard_pages(guide, &pages, repo, &mut summary);
    }
    check_features(label, &features, options, &mut summary);
    summary
}

/// The listed pages, in run order; every listed name must be a page (§4.7 rule 7).
fn selected(pages: Vec<Listed>, wanted: &[String]) -> Result<Vec<Listed>, String> {
    if wanted.is_empty() {
        return Ok(pages);
    }
    if let Some(missing) = wanted
        .iter()
        .find(|name| !pages.iter().any(|page| &page.name == *name))
    {
        return Err(format!("the run names page {missing}, which the guide does not have"));
    }
    Ok(pages
        .into_iter()
        .filter(|page| wanted.contains(&page.name))
        .collect())
}

fn read_features(guide: &Path) -> Result<Features, String> {
    let text = fs::read_to_string(guide.join("features.toml"))
        .map_err(|error| format!("features.toml cannot be read: {error}"))?;
    features::read(&text)
}

/// One English page: its name, as reports, `continues` and mirrors use it (the path from the guide folder,
/// without `.md`, such as `05-one-shop` or `compare/weather`), and its file.
pub struct Listed {
    pub name: String,
    pub path: PathBuf,
}

/// The folder of comparison pages inside the guide folder (§1.1, rule 4).
const COMPARE: &str = "compare";

/// Every English page in run order: the guide folder's pages in file-name order, then the comparison
/// folder's, in file-name order (§1.1). In each folder, a page is a Markdown file other than the index
/// (`README.md` and its mirror `README.zh-CN.md`) and mirrors. A mirror without its English page is a
/// failure (§1.2).
fn english_pages(guide: &Path) -> Result<Vec<Listed>, String> {
    let mut pages = pages_in(guide, "")?;
    let compare = guide.join(COMPARE);
    if compare.is_dir() {
        pages.extend(pages_in(&compare, &format!("{COMPARE}/"))?);
    }
    Ok(pages)
}

/// The English pages directly in `folder`, each named with `prefix` before its file stem.
fn pages_in(folder: &Path, prefix: &str) -> Result<Vec<Listed>, String> {
    let entries = fs::read_dir(folder)
        .map_err(|error| format!("{} cannot be read: {error}", folder.display()))?;
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
            mirrors.push(format!("{prefix}{stem}"));
        } else if let Some(stem) = name.strip_suffix(".md") {
            english.push(Listed {
                name: format!("{prefix}{stem}"),
                path,
            });
        }
    }
    english.sort_by(|left, right| left.name.cmp(&right.name));
    for mirror in &mirrors {
        if !english.iter().any(|page| &page.name == mirror) {
            return Err(format!("{mirror}.zh-CN.md has no English page {mirror}.md"));
        }
    }
    Ok(english)
}

/// The guide's index and its mirror: read by people, never executed, and not pages (§1). The comparison
/// folder has its own index, under the same names.
const INDEX: [&str; 2] = ["README.md", "README.zh-CN.md"];


/// Runs every page's executed blocks in order. A page that continues an earlier one uses its scratch folder
/// and inherits its pending state (§5, §8.1). Returns whether `after_page` stopped the run.
fn run_pages(
    label: &str,
    pages: &[Listed],
    features: &Features,
    repo: &Path,
    options: &RunOptions,
    after_page: &mut AfterPage,
    summary: &mut Summary,
) -> bool {
    // The reader's home folder, one per run: chapter 01 clones MineWorld into it, and later pages copy from
    // that clone (§4.6). A cold-start run uses the real one (§4.7 rule 1).
    let scratch_home = match options.home {
        Some(_) => None,
        None => Some(Scratch::empty(options.scratch_root, format!("{label}-home"))),
    };
    let home = options
        .home
        .or(scratch_home.as_ref().map(Scratch::path))
        .expect("a home folder");
    let environment = Environment {
        features,
        repo,
        home,
        binary: options.binary,
        mode: options.mode,
    };
    let mut folders: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut carries: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut kept: Vec<Scratch> = Vec::new();
    let mut mutated = false;
    for Listed { name, path } in pages {
        let name = name.clone();
        let mut source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                summary
                    .checks
                    .push((name, Verdict::Fail(error.to_string())));
                continue;
            }
        };
        if let Some(mutation) = options.mutation.filter(|mutation| mutation.page == name) {
            match mutate(&source, mutation) {
                Ok(changed) => {
                    source = changed;
                    mutated = true;
                }
                Err(reason) => {
                    summary.checks.push((
                        format!("mutation {}", mutation.name),
                        Verdict::Fail(reason),
                    ));
                    return true;
                }
            }
        }
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
                let scratch =
                    Scratch::empty(options.scratch_root, format!("{label}-{}", name.replace('/', "-")));
                let folder = scratch.path().to_path_buf();
                kept.push(scratch);
                (folder, None)
            }
        };
        let began = Instant::now();
        let (reports, carry) = run_page(&name, &parsed, &folder, &environment, carried);
        summary.timings.push((name.clone(), began.elapsed()));
        record(reports, summary);
        let checks = after_page(&name, &folder);
        let stop = checks.iter().any(|(_, verdict)| *verdict != Verdict::Pass);
        summary.checks.extend(checks);
        if stop {
            return true;
        }
        folders.insert(name.clone(), folder);
        carries.insert(name, carry);
    }
    if let Some(mutation) = options.mutation.filter(|_| !mutated) {
        summary.checks.push((
            format!("mutation {}", mutation.name),
            Verdict::Fail(format!("page {} is not in this run", mutation.page)),
        ));
        return true;
    }
    false
}

/// `source` with every occurrence of the mutation's text replaced (so a step written for both shells is
/// mutated for both); a page that does not hold the text fails the run, so a mutation can never pass by
/// changing nothing.
fn mutate(source: &str, mutation: &Mutation) -> Result<String, String> {
    let normalised = source.replace("\r\n", "\n");
    if !normalised.contains(&mutation.old) {
        return Err(format!(
            "page {} does not hold the text {:?}",
            mutation.page, mutation.old
        ));
    }
    Ok(normalised.replace(&mutation.old, &mutation.new))
}

/// Counts each block, lists pending blocks by page, step and feature, and records each failure by name.
fn record(reports: Vec<Report>, summary: &mut Summary) {
    for report in reports {
        let step = report.step.as_deref().unwrap_or("(no step)");
        let place = format!("page {} line {} step {step}", report.page, report.line);
        summary.blocks += 1;
        let word = match &report.outcome {
            Outcome::Passed => "PASS",
            Outcome::Pending(_) => "PENDING",
            Outcome::OtherOs(_) => "OTHER OS",
            Outcome::Failed(_) => "FAIL",
        };
        summary.block_times.push(BlockTime {
            page: report.page.clone(),
            step: step.to_owned(),
            line: report.line,
            elapsed: report.elapsed,
            outcome: word,
        });
        match report.outcome {
            Outcome::Passed => summary.passed += 1,
            Outcome::Pending(feature) => {
                summary
                    .not_run
                    .push(format!("{} | {step} | {feature}", report.page));
                summary
                    .pending
                    .push(format!("{place}: waits for {feature}"));
            }
            Outcome::OtherOs(shell) => summary.other_os.push(format!(
                "{place}: a {shell} block, run on that shell's systems"
            )),
            Outcome::Failed(reason) => summary.checks.push((place, Verdict::Fail(reason))),
        }
    }
}

/// §1.2, §7 and §8.2–8.3: each English page with its mirror, and its links.
fn guard_pages(guide: &Path, pages: &[Listed], repo: &Path, summary: &mut Summary) {
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
    for Listed { name, path } in pages {
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


/// §9 rule 3: the flags against what the binary itself shows (§9.1). The evidence is asked for once, in a
/// scratch folder of its own, so the guide folder is never written.
fn check_features(label: &str, features: &Features, options: &RunOptions, summary: &mut Summary) {
    let scratch = Scratch::empty(options.scratch_root, format!("{label}-probes"));
    let binary = options.binary;
    let build = Build {
        packs: ask(binary, &scratch, &["packs", "list"]),
        models: installed_models(binary, &scratch),
        reference: generated_reference(binary, &scratch),
    };
    for (feature, verdict) in features::against_build(features, &build) {
        summary.checks.push((format!("feature {feature}"), verdict));
    }
}

/// What `mineworld ARGS` printed, when it exited 0.
fn ask(binary: &Path, scratch: &Path, arguments: &[&str]) -> Result<String, String> {
    let words: Vec<String> = std::iter::once("mineworld")
        .chain(arguments.iter().copied())
        .map(str::to_owned)
        .collect();
    match command::run(binary, scratch, &words)? {
        (output, 0) => Ok(output),
        (output, code) => Err(format!(
            "`{}` exited with {code}:\n{output}",
            words.join(" ")
        )),
    }
}

/// The decision models this build installs. `mineworld decisions` refuses a seat model no build installs and
/// lists the installed ones (`docs/MODULE_SPEC.md` §4.1 rule 1), so the probe asks for one in a world it
/// creates and reads the list.
fn installed_models(binary: &Path, scratch: &Path) -> Result<Vec<String>, String> {
    ask(binary, scratch, &["create", "probe"])?;
    let words = [
        "mineworld",
        "decisions",
        "probe",
        "--seat-model",
        "first=no-such-model",
    ]
    .map(str::to_owned);
    let (output, _) = command::run(binary, scratch, &words)?;
    let marker = "it installs: ";
    let listed = output
        .lines()
        .find_map(|line| line.split_once(marker).map(|(_, models)| models))
        .ok_or_else(|| {
            format!(
                "`{}` printed no {marker:?} list:\n{output}",
                words.join(" ")
            )
        })?;
    Ok(listed
        .split(',')
        .map(|model| model.trim().to_owned())
        .collect())
}

/// Every page `mineworld reference --out` writes, by file name.
fn generated_reference(binary: &Path, scratch: &Path) -> Result<BTreeMap<String, String>, String> {
    ask(binary, scratch, &["reference", "--out", "reference"])?;
    let folder = scratch.join("reference");
    let mut pages = BTreeMap::new();
    let entries = fs::read_dir(&folder)
        .map_err(|error| format!("{} cannot be read: {error}", folder.display()))?;
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().is_some_and(|extension| extension == "md") {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_owned();
            let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
            pages.insert(name, text);
        }
    }
    Ok(pages)
}
