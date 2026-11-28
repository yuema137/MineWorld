//! The runner's cold-start mode (`docs/GUIDE_FORMAT.md` §4.7, pr-25g §11): the binary as a parameter, the
//! stranger's shell environment, the `cold-start` directive, the page filter, timings, the configuration's
//! loader and the committed mutations. The run itself is the workflow's (`cold_start`, ignored).

use std::path::Path;

use super::cold_start;
use super::command::{self, Rules};
use super::features::Features;
use super::page::{Directive, parse};
use super::page_run::{COLD_START, Environment, Outcome, run_page};
use super::report::Verdict;
use super::run_guide::{self, Mutation, RunOptions};
use super::{failures, page_and_mirror, repo_with, run_here};

#[test]
fn a_cold_start_shell_block_sees_the_real_home_and_none_of_the_repository_runs_rules() {
    // §4.7 rule 1–2: the process's own home, and no Cargo target, clone redirect or LFS skip.
    let scratch = mineworld_test_support::scratch!(empty "guide-cold-env");
    let home = mineworld_test_support::scratch!(empty "guide-cold-env-home");
    let (family, text) = if cfg!(windows) {
        (
            super::page::Family::PowerShell,
            "Set-Content -NoNewline -Path seen.txt -Value \"$env:USERPROFILE|$env:CARGO_TARGET_DIR|$env:GIT_CONFIG_COUNT|$env:GIT_LFS_SKIP_SMUDGE\"",
        )
    } else {
        (
            super::page::Family::Posix,
            "printf '%s|%s|%s|%s' \"$HOME\" \"$CARGO_TARGET_DIR\" \"$GIT_CONFIG_COUNT\" \"$GIT_LFS_SKIP_SMUDGE\" > seen.txt",
        )
    };
    command::shell(&scratch, &home, &scratch, family, text, Rules::ColdStart)
        .expect("the block runs");
    let seen = std::fs::read_to_string(scratch.join("seen.txt")).expect("the block wrote");
    let real = std::env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).expect("a home");
    assert_eq!(seen, format!("{real}|||"), "cold-start mode");

    // The same block in the repository's mode sees the run's home and every rule of §4.6.
    command::shell(&scratch, &home, &scratch, family, text, Rules::Repository)
        .expect("the block runs");
    let seen = std::fs::read_to_string(scratch.join("seen.txt")).expect("the block wrote");
    let fields: Vec<&str> = seen.split('|').collect();
    assert_eq!(fields[0], home.to_string_lossy(), "{seen}");
    assert!(fields[1].ends_with("guide-build"), "{seen}");
    assert_eq!(&fields[2..], ["2", "1"], "{seen}");
}

#[test]
fn a_console_block_runs_the_binary_it_is_given_and_a_missing_one_fails_naming_its_path() {
    let page = "```console\n$ mineworld --version\nmineworld [..]\n```\n";
    let scratch = mineworld_test_support::scratch!(empty "guide-cold-binary");
    let features = Features::new();
    let missing = scratch.join("no-such-mineworld");
    let environment = Environment {
        binary: &missing,
        ..Environment::repository(&features, &scratch, &scratch)
    };
    let (reports, _) = run_page(
        "binary",
        &parse(page).expect("parses"),
        &scratch,
        &environment,
        None,
    );
    let Outcome::Failed(reason) = &reports[0].outcome else {
        panic!("a missing binary fails the block: {reports:#?}");
    };
    assert!(reason.contains("no-such-mineworld"), "{reason}");

    // A copy of the real binary under another name is what runs when it is named.
    let copy = scratch.join(format!("reader-mineworld{}", std::env::consts::EXE_SUFFIX));
    std::fs::copy(command::BINARY, &copy).expect("the binary is copied");
    let environment = Environment {
        binary: &copy,
        ..Environment::repository(&features, &scratch, &scratch)
    };
    let (reports, _) = run_page(
        "binary",
        &parse(page).expect("parses"),
        &scratch,
        &environment,
        None,
    );
    assert_eq!(reports[0].outcome, Outcome::Passed, "{reports:#?}");
}

#[test]
fn a_cold_start_block_is_listed_in_the_default_run_and_does_not_stop_the_blocks_after_it() {
    let page = "### Step 1 — Install\n\n<!-- guide: cold-start -->\n\n```sh\nexit 1\n```\n\n\
<!-- guide: cold-start -->\n\n```powershell\nexit 1\n```\n\n```console\n$ mineworld --version\nmineworld [..]\n```\n";
    let parsed = parse(page).expect("a cold-start shell block parses");
    assert_eq!(parsed.blocks[0].directives, [Directive::ColdStart]);
    let scratch = mineworld_test_support::scratch!(empty "guide-cold-directive");
    let reports = run_here("cold", page, &scratch, &Features::new());
    // Both shells' blocks are listed the same way on every system, so the committed list is one list.
    assert_eq!(reports[0].outcome, Outcome::Pending(COLD_START.to_owned()));
    assert_eq!(reports[1].outcome, Outcome::Pending(COLD_START.to_owned()));
    assert_eq!(reports[2].outcome, Outcome::Passed, "{reports:#?}");

    for refused in [
        "<!-- guide: cold-start -->\n\n```console\n$ mineworld --version\n```\n",
        "<!-- guide: cold-start -->\n\n```yaml file=a.yaml\nk: 1\n```\n",
    ] {
        let reason = parse(refused).expect_err("cold-start qualifies shell blocks only");
        assert!(reason.contains("ColdStart"), "{reason}");
    }
}

#[test]
fn in_cold_start_mode_a_cold_start_block_runs() {
    let (shell, other) = if cfg!(windows) {
        ("powershell", "sh")
    } else {
        ("sh", "powershell")
    };
    let page = format!(
        "<!-- guide: cold-start -->\n\n```{shell}\nmkdir installed\n```\n\n<!-- guide: cold-start -->\n\n```{other}\nexit 1\n```\n"
    );
    let scratch = mineworld_test_support::scratch!(empty "guide-cold-runs");
    let features = Features::new();
    let environment = Environment {
        rules: Rules::ColdStart,
        ..Environment::repository(&features, &scratch, &scratch)
    };
    let (reports, _) = run_page(
        "runs",
        &parse(&page).expect("parses"),
        &scratch,
        &environment,
        None,
    );
    assert_eq!(reports[0].outcome, Outcome::Passed, "{reports:#?}");
    assert!(
        matches!(reports[1].outcome, Outcome::OtherOs(_)),
        "{reports:#?}"
    );
    assert!(scratch.join("installed").is_dir());
}

/// A two-page mock guide: `01-a` makes a world, `02-b` makes another; neither continues the other.
fn two_pages(label: &str) -> mineworld_test_support::Scratch {
    let first = "### Step 1.1 — Make\n\n```console\n$ mineworld create first\n...\n```\n";
    let second = "### Step 2.1 — Make\n\n```console\n$ mineworld create second\n...\n```\n";
    let mut files = page_and_mirror("01-a", first);
    files.extend(page_and_mirror("02-b", second));
    repo_with(label, files)
}

fn options<'a>(pages: &'a [String], mutation: Option<&'a Mutation>) -> RunOptions<'a> {
    RunOptions {
        pages,
        mutation,
        ..RunOptions::repository()
    }
}

#[test]
fn a_page_list_runs_only_those_pages_and_every_page_run_has_a_time() {
    let repo = two_pages("guide-cold-pages");
    let guide = repo.join("docs/guide");
    let only = vec!["02-b".to_owned()];
    let summary = run_guide::run_with(
        "cold-pages",
        &guide,
        &repo,
        &options(&only, None),
        &mut |_, _| Vec::new(),
    );
    assert!(summary.no_failures(), "{}", summary.render());
    assert_eq!(summary.passed, 1, "{}", summary.render());
    let timed: Vec<&str> = summary
        .timings
        .iter()
        .map(|(page, _)| page.as_str())
        .collect();
    assert_eq!(timed, ["02-b"]);
    assert!(
        summary.render().contains("timings: 02-b "),
        "{}",
        summary.render()
    );

    let unknown = vec!["03-c".to_owned()];
    let summary = run_guide::run_with(
        "cold-pages-unknown",
        &guide,
        &repo,
        &options(&unknown, None),
        &mut |_, _| Vec::new(),
    );
    assert!(
        failures(&summary).contains("page 03-c"),
        "{}",
        summary.render()
    );
}

#[test]
fn a_check_after_a_page_that_does_not_pass_stops_the_run_there() {
    let repo = two_pages("guide-cold-stop");
    let guide = repo.join("docs/guide");
    let mut seen = Vec::new();
    let summary = run_guide::run_with(
        "cold-stop",
        &guide,
        &repo,
        &options(&[], None),
        &mut |page, _| {
            seen.push(page.to_owned());
            vec![(
                "cold-start: gate".to_owned(),
                Verdict::Inconclusive("not proved".to_owned()),
            )]
        },
    );
    assert_eq!(seen, ["01-a"], "the run stopped after the first page");
    assert_eq!(summary.passed, 1, "{}", summary.render());
}

#[test]
fn a_mutation_changes_only_its_page_in_memory_and_one_that_changes_nothing_fails() {
    let repo = two_pages("guide-cold-mutation");
    let guide = repo.join("docs/guide");
    let before = std::fs::read_to_string(guide.join("02-b.md")).expect("the page");
    let wrong = Mutation {
        name: "wrong".to_owned(),
        page: "02-b".to_owned(),
        old: "create second\n...".to_owned(),
        new: "create second\nno such line".to_owned(),
    };
    let summary = run_guide::run_with(
        "cold-mutation",
        &guide,
        &repo,
        &options(&[], Some(&wrong)),
        &mut |_, _| Vec::new(),
    );
    let found = failures(&summary);
    assert!(found.contains("page 02-b"), "{found}");
    assert!(!found.contains("page 01-a"), "{found}");
    assert_eq!(
        std::fs::read_to_string(guide.join("02-b.md")).expect("the page"),
        before,
        "never on disk"
    );

    let absent = Mutation {
        old: "text the page does not hold".to_owned(),
        ..wrong
    };
    let summary = run_guide::run_with(
        "cold-mutation-absent",
        &guide,
        &repo,
        &options(&[], Some(&absent)),
        &mut |_, _| Vec::new(),
    );
    assert!(
        failures(&summary).contains("does not hold the text"),
        "{}",
        summary.render()
    );
}

#[test]
fn the_committed_mutations_apply_to_this_guide_and_an_unknown_name_is_refused() {
    let unknown = cold_start::mutation("no-such-mutation").expect_err("an unknown name");
    assert!(
        unknown.contains("wrong-build"),
        "the refusal lists the committed ones: {unknown}"
    );
    let guide = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/guide");
    for name in ["wrong-build", "wrong-expected"] {
        let mutation = cold_start::mutation(name).expect("committed");
        let page = std::fs::read_to_string(guide.join(format!("{}.md", mutation.page)))
            .expect("the page")
            .replace("\r\n", "\n");
        assert!(
            page.contains(&mutation.old),
            "{name}: the page holds the text it changes"
        );
    }
}

#[test]
fn the_cold_start_entry_refuses_to_run_without_its_configuration() {
    let reason = cold_start::load(None).expect_err("no variable");
    assert!(
        reason.contains("MINEWORLD_COLD_START") && reason.contains("§4.7"),
        "{reason}"
    );
    let reason = cold_start::load(Some("/no/such/cold-start.toml".into())).expect_err("no file");
    assert!(reason.contains("cannot be read"), "{reason}");
    let reason = cold_start::parse("layer = \"c\"").expect_err("no such layer");
    assert!(
        reason.contains("phase") || reason.contains("layer"),
        "{reason}"
    );
}

#[test]
fn the_committed_cold_start_expectation_reads_for_both_layers() {
    for layer in [cold_start::Layer::A, cold_start::Layer::B] {
        let (not_run, minimum) = cold_start::expected(layer).expect("expected_cold_start.toml");
        assert!(minimum > 0, "{layer:?}");
        assert!(
            not_run
                .iter()
                .all(|entry| entry.ends_with("| operator-only")),
            "in cold-start mode only operator-only blocks are not run: {not_run:#?}"
        );
    }
}
