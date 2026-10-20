//! The executable guide (`docs/GUIDE_FORMAT.md`, ARC-90, DEP-50): the parts of a page a reader follows are
//! run against the real `mineworld` binary, so a page that stops working is a red test.
//!
//! This file holds the tests; the reading, writing and checking live in `guide/`.

#[path = "guide/command.rs"]
mod command;
#[path = "guide/features.rs"]
mod features;
#[path = "guide/files.rs"]
mod files;
#[path = "guide/mirror.rs"]
mod mirror;
#[path = "guide/page.rs"]
mod page;
#[path = "guide/page_run.rs"]
mod page_run;
#[path = "guide/report.rs"]
mod report;
#[path = "guide/run_guide.rs"]
mod run_guide;

use std::path::{Path, PathBuf};

use features::Features;
use mineworld_test_support::Scratch;
use page::{Body, Directive, parse};
use page_run::{Environment, Outcome, Report, run_page};
use report::{Summary, Verdict};

/// A page with a step, a directive, a file block, a console block and a shown-only block.
const PAGE: &str = "# A page\n\
\n\
### Step 1.1 — Make the place\n\
\n\
<!-- guide: requires calendar -->\n\
\n\
```yaml file=shop/places/home.yaml\n\
tags: [home]\n\
```\n\
\n\
```console\n\
$ mineworld validate shop\n\
Corner Shop (shop)\n\
...\n\
```\n\
\n\
```text\n\
not run by the runner\n\
```\n";

#[test]
fn a_page_yields_its_executed_blocks_in_order_with_their_step_and_directives() {
    let page = parse(PAGE).expect("the page parses");
    assert_eq!(page.continues, None);
    assert_eq!(
        page.blocks.len(),
        2,
        "the shown-only block is not executed: {page:?}"
    );

    let file = &page.blocks[0];
    assert_eq!(file.line, 7);
    assert_eq!(file.step.as_deref(), Some("Step 1.1 — Make the place"));
    assert_eq!(
        file.directives,
        [Directive::Requires("calendar".to_owned())]
    );
    assert_eq!(
        file.body,
        Body::File {
            path: "shop/places/home.yaml".to_owned(),
            text: "tags: [home]\n".to_owned(),
        }
    );

    let console = &page.blocks[1];
    assert!(console.directives.is_empty());
    assert_eq!(
        console.body,
        Body::Console {
            exit: 0,
            lines: vec![
                "$ mineworld validate shop".to_owned(),
                "Corner Shop (shop)".to_owned(),
                "...".to_owned(),
            ],
        }
    );
}

#[test]
fn crlf_pages_are_read_as_lf_so_the_same_blocks_run_on_every_checkout() {
    let crlf = PAGE.replace('\n', "\r\n");
    let from_crlf = parse(&crlf).expect("the CRLF page parses");
    let from_lf = parse(PAGE).expect("the LF page parses");
    assert_eq!(from_crlf, from_lf);
}

#[test]
fn an_info_string_token_the_guide_does_not_list_fails_the_page_naming_its_line() {
    let page = "text\n\n```yaml file=a.yaml extra\nx: 1\n```\n";
    let error = parse(page).expect_err("an unlisted token is refused");
    assert!(error.contains("line 3"), "{error}");
    assert!(error.contains("\"extra\""), "{error}");
}

#[test]
fn tilde_fences_and_indented_blocks_are_refused_rather_than_skipped() {
    let tilde = "~~~yaml file=a.yaml\nx: 1\n~~~\n";
    assert!(parse(tilde).expect_err("a tilde fence").contains("tilde"));

    let indented = "Text.\n\n    $ mineworld validate x\n";
    assert!(
        parse(indented)
            .expect_err("an indented block")
            .contains("indented")
    );
}

#[test]
fn a_directive_with_no_block_to_qualify_fails_the_page() {
    let waiting = "<!-- guide: requires calendar -->\n\n```text\nshown only\n```\n";
    assert!(
        parse(waiting)
            .expect_err("a directive on a shown block")
            .contains("no executed block")
    );

    // `copy` stood in for a shown command until the operator's ruling of 2026-10-18; it is no directive now.
    let copy = "<!-- guide: copy a b -->\n\n```console\n$ mineworld --version\n```\n";
    assert!(
        parse(copy)
            .expect_err("copy is not a directive")
            .contains("not one the runner knows")
    );
}

#[test]
fn an_unknown_directive_or_one_with_the_wrong_arguments_fails_the_page() {
    let unknown = "<!-- guide: teleport somewhere -->\n";
    assert!(parse(unknown).expect_err("unknown").contains("teleport"));

    let arguments = "<!-- guide: requires -->\n";
    assert!(parse(arguments).is_err());
}

#[test]
fn continues_is_given_once_and_before_the_first_block() {
    let first = "<!-- guide: continues 01-start -->\n\n```console\n$ mineworld validate x\n```\n";
    assert_eq!(
        parse(first)
            .expect("continues before the block")
            .continues
            .as_deref(),
        Some("01-start")
    );

    let late = "```console\n$ mineworld validate x\n```\n\n<!-- guide: continues 01-start -->\n";
    assert!(
        parse(late)
            .expect_err("continues after a block")
            .contains("before the first block")
    );
}

#[test]
fn a_console_block_needs_a_command_and_no_output_before_it() {
    let no_command = "```console\njust output\n```\n";
    assert!(
        parse(no_command)
            .expect_err("no command")
            .contains("no command line")
    );

    let output_first = "```console\nout\n$ mineworld validate x\n```\n";
    assert!(
        parse(output_first)
            .expect_err("output first")
            .contains("before the first command")
    );

    let exit = "```console exit=3\n$ mineworld validate missing\n```\n";
    let Body::Console { exit, .. } = &parse(exit).expect("exit parses").blocks[0].body else {
        panic!("a console block");
    };
    assert_eq!(*exit, 3);

    let on_yaml = "```yaml exit=3\nx: 1\n```\n";
    assert!(
        parse(on_yaml)
            .expect_err("exit on yaml")
            .contains("console blocks only")
    );
}

#[test]
fn a_block_path_outside_the_scratch_folder_is_refused_by_the_page_reader() {
    for path in ["../x", "/abs/x", "C:/x", "a\\b", "a//b", "a/./b", "a/"] {
        let page = format!("```yaml file={path}\nx: 1\n```\n");
        assert!(parse(&page).is_err(), "{path} must be refused");
    }
}

#[test]
fn a_file_block_writes_its_bytes_and_its_parent_folders_and_nothing_outside() {
    let scratch = mineworld_test_support::scratch!("guide-files-write");
    files::write(&scratch, "shop/places/home.yaml", "tags: [home]\n").expect("written");
    let written = std::fs::read_to_string(scratch.join("shop/places/home.yaml")).expect("read");
    assert_eq!(written, "tags: [home]\n");

    assert!(files::write(&scratch, "../escaped.yaml", "x").is_err());
    assert!(
        !scratch
            .parent()
            .expect("a parent")
            .join("escaped.yaml")
            .exists()
    );
}

#[cfg(unix)]
#[test]
fn a_symbolic_link_already_in_the_scratch_folder_is_not_followed() {
    let scratch = mineworld_test_support::scratch!(empty "guide-files-link");
    let outside = scratch
        .parent()
        .expect("a parent")
        .join("guide-files-link-outside");
    std::fs::create_dir_all(&outside).expect("made");
    std::os::unix::fs::symlink(&outside, scratch.join("out")).expect("linked");
    let error = files::write(&scratch, "out/x.yaml", "x").expect_err("a link on the way");
    assert!(error.contains("symbolic link"), "{error}");
    assert!(!outside.join("x.yaml").exists());
    let _ = std::fs::remove_dir_all(&outside);
}

#[test]
fn features_toml_reads_named_flags_and_refuses_anything_else() {
    let read = features::read("# merged\n[features]\ncalendar = true\nhappenings = false # WE-a\n")
        .expect("valid");
    assert_eq!(read.get("calendar"), Some(&true));
    assert_eq!(read.get("happenings"), Some(&false));

    assert!(
        features::read("[features]\nCalendar = true\n").is_err(),
        "capitals are refused"
    );
    assert!(
        features::read("[features]\ncalendar = \"yes\"\n").is_err(),
        "a string is not a flag"
    );
    assert!(
        features::read("[other]\ncalendar = true\n").is_err(),
        "another table is refused"
    );
    assert!(
        features::read("calendar = true\n").is_err(),
        "a top-level flag is refused"
    );
    assert!(
        features::read("").is_err(),
        "a file with no [features] is refused"
    );
}

/// Runs one page's source in `scratch`, with no features beyond the given ones and no committed folders.
fn run_here(name: &str, source: &str, scratch: &Path, features: &Features) -> Vec<Report> {
    let environment = Environment {
        features,
        repo: scratch,
        home: scratch,
    };
    run_page(
        name,
        &parse(source).expect("the page parses"),
        scratch,
        &environment,
        None,
    )
    .0
}

/// The fixture page, read from the folder beside this test.
fn fixture() -> page::Page {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/guide/fixture.md"
    ))
    .expect("the fixture page is read");
    parse(&source).expect("the fixture page parses")
}

#[test]
fn the_fixture_page_runs_green_against_the_real_binary() {
    let scratch = mineworld_test_support::scratch!(empty "guide-fixture");
    let environment = Environment {
        features: &Features::new(),
        repo: &scratch,
        home: &scratch,
    };
    let (reports, _) = run_page("fixture", &fixture(), &scratch, &environment, None);
    assert_eq!(reports.len(), 3, "{reports:#?}");
    for report in &reports {
        assert_eq!(report.outcome, Outcome::Passed, "{report:#?}");
    }
    assert!(
        scratch.join("seaside/world.yaml").is_file(),
        "the world was created in the scratch folder"
    );
}

#[test]
fn a_wrong_expected_line_fails_its_block_and_names_the_step_and_the_difference() {
    let page = "```console\n$ mineworld create seaside\n```\n\n\
### Step 2 — Check\n\n```console\n$ mineworld validate seaside\nseaside (other)\n```\n";
    // The page's second block is a validate of a world the first block created: its output is the only mismatch.
    let scratch = mineworld_test_support::scratch!(empty "guide-wrong-output");
    let reports = run_here("wrong", page, &scratch, &Features::new());
    let Outcome::Failed(reason) = &reports[1].outcome else {
        panic!("the mismatched block fails: {reports:#?}");
    };
    assert_eq!(reports[1].step.as_deref(), Some("Step 2 — Check"));
    assert!(reason.contains("printed other output"), "{reason}");
    assert!(
        reason.contains("seaside (other)"),
        "the difference is shown: {reason}"
    );
}

#[test]
fn a_shell_pipe_in_a_command_block_fails_the_block_and_nothing_is_run() {
    let piped = "```console\n$ mineworld validate seaside | more\nx\n```\n";
    let scratch = mineworld_test_support::scratch!(empty "guide-pipe");
    let reports = run_here("piped", piped, &scratch, &Features::new());
    let Outcome::Failed(reason) = &reports[0].outcome else {
        panic!("the pipe is refused: {reports:#?}");
    };
    assert!(reason.contains("shell syntax"), "{reason}");
    assert_eq!(
        std::fs::read_dir(&*scratch).expect("read").count(),
        0,
        "no command ran"
    );
}

#[test]
fn an_exit_code_other_than_the_one_the_block_expects_fails_it() {
    let scratch = mineworld_test_support::scratch!(empty "guide-exit");
    let wrong = "```console\n$ mineworld validate nothing-here\n```\n";
    let reports = run_here("exit", wrong, &scratch, &Features::new());
    let Outcome::Failed(reason) = &reports[0].outcome else {
        panic!("exit 1 against an expected 0 fails: {reports:#?}");
    };
    assert!(reason.contains("exited with 1"), "{reason}");

    let expected = "```console exit=1\n$ mineworld validate nothing-here\n[mineworld] nothing-here is not a World Pack directory\n```\n";
    let reports = run_here("exit", expected, &scratch, &Features::new());
    assert_eq!(reports[0].outcome, Outcome::Passed, "{reports:#?}");
}

#[test]
fn a_block_waiting_for_an_unmerged_feature_is_pending_and_so_is_the_rest_of_its_page() {
    let page = "### Step 1 — Wait\n\n<!-- guide: requires happenings -->\n\n```console\n$ mineworld validate x\n```\n\n\
### Step 2 — After\n\n```console\n$ mineworld validate x\n```\n";
    let scratch = mineworld_test_support::scratch!(empty "guide-pending");
    let features = features::read("[features]\nhappenings = false\n").expect("features");
    let environment = Environment {
        features: &features,
        repo: &scratch,
        home: &scratch,
    };
    let (reports, carried) = run_page(
        "waiting",
        &parse(page).expect("parses"),
        &scratch,
        &environment,
        None,
    );
    assert_eq!(
        reports[0].outcome,
        Outcome::Pending("happenings".to_owned())
    );
    assert_eq!(
        reports[1].outcome,
        Outcome::Pending("happenings".to_owned()),
        "chained blocks wait too"
    );
    assert_eq!(
        carried.as_deref(),
        Some("happenings"),
        "a page that continues this one waits too"
    );
}

#[test]
fn a_directive_naming_a_feature_that_is_not_in_features_toml_fails_the_block() {
    let page = "<!-- guide: requires teleporting -->\n\n```console\n$ mineworld validate x\n```\n";
    let scratch = mineworld_test_support::scratch!(empty "guide-unknown-feature");
    let reports = run_here("unknown", page, &scratch, &Features::new());
    let Outcome::Failed(reason) = &reports[0].outcome else {
        panic!("an unknown feature fails: {reports:#?}");
    };
    assert!(reason.contains("teleporting"), "{reason}");
}

#[test]
fn a_shell_block_runs_on_its_own_systems_in_the_scratch_folder_and_is_listed_elsewhere() {
    // The same step for both shells, as a page writes it (§4.6): each runs where its shell is the system's.
    let page = "### Step 1 — Copy\n\n```yaml file=a/x.yaml\nk: 1\n```\n\n\
```sh\nmkdir b\ncp -R a b/\n```\n\n\
```powershell\nmkdir b\nCopy-Item -Recurse a b\n```\n\n\
```console\n$ mineworld --version\nmineworld [..]\n```\n";
    let scratch = mineworld_test_support::scratch!(empty "guide-shell");
    let reports = run_here("shell", page, &scratch, &Features::new());
    let (here, there) = if cfg!(windows) { (2, 1) } else { (1, 2) };
    assert_eq!(reports[here].outcome, Outcome::Passed, "{reports:#?}");
    assert!(
        matches!(reports[there].outcome, Outcome::OtherOs(_)),
        "{reports:#?}"
    );
    assert_eq!(
        std::fs::read_to_string(scratch.join("b/a/x.yaml")).expect("copied by the shell"),
        "k: 1\n"
    );
}

#[test]
fn a_shell_block_that_fails_fails_its_step_with_what_it_printed() {
    let page = if cfg!(windows) {
        "```powershell\nCopy-Item -Recurse missing elsewhere\n```\n"
    } else {
        "```sh\ncp -R missing elsewhere\n```\n"
    };
    let scratch = mineworld_test_support::scratch!(empty "guide-shell-fails");
    let reports = run_here("shell-fails", page, &scratch, &Features::new());
    let Outcome::Failed(reason) = &reports[0].outcome else {
        panic!("a failing command fails the block: {reports:#?}");
    };
    assert!(reason.contains("block exited with"), "{reason}");
    assert!(
        reason.contains("missing"),
        "what it printed is shown: {reason}"
    );
}

#[test]
fn a_step_with_two_files_or_without_a_command_is_refused_by_the_page_reader() {
    let two_files = "### Step 1 — Two\n\n```yaml file=a.yaml\nk: 1\n```\n\n```yaml file=b.yaml\nk: 2\n```\n\n\
```console\n$ mineworld --version\n```\n";
    assert!(
        parse(two_files)
            .expect_err("two files in one step")
            .contains("§10 rule 1")
    );
    let no_command = "### Step 1 — Look\n\n```sh\nls\n```\n\n**You should see** a folder.\n\n### Step 2 — Next\n\n\
```console\n$ mineworld --version\n```\n";
    let reason = parse(no_command).expect_err("a step with no console block");
    assert!(
        reason.contains("§10 rule 3") && reason.contains("Step 1"),
        "{reason}"
    );
    let handed_over =
        "### Step 1 — Play\n\n<!-- guide: operator-only -->\n\n```sh\n./client\n```\n";
    assert!(
        parse(handed_over).is_ok(),
        "an operator-only block is the operator's check"
    );
}

#[test]
fn equals_holds_when_the_scratch_folder_matches_the_committed_one_and_fails_on_one_byte() {
    let repo = mock_repo(
        "guide-equals",
        &[("docs/guide/examples/w/x.yaml", "a: 1\n")],
    );
    let matching =
        "<!-- guide: equals w docs/guide/examples/w -->\n\n```yaml file=w/x.yaml\na: 1\n```\n";
    let differing =
        "<!-- guide: equals w docs/guide/examples/w -->\n\n```yaml file=w/x.yaml\na: 2\n```\n";
    let scratch = mineworld_test_support::scratch!(empty "guide-equals-run");
    let environment = Environment {
        features: &Features::new(),
        repo: &repo,
        home: &repo,
    };
    let (ok, _) = run_page(
        "ok",
        &parse(matching).expect("parses"),
        &scratch,
        &environment,
        None,
    );
    assert_eq!(ok[0].outcome, Outcome::Passed, "{ok:#?}");

    let scratch = mineworld_test_support::scratch!(empty "guide-equals-differs");
    let (bad, _) = run_page(
        "bad",
        &parse(differing).expect("parses"),
        &scratch,
        &environment,
        None,
    );
    let Outcome::Failed(reason) = &bad[0].outcome else {
        panic!("a differing byte fails: {bad:#?}");
    };
    assert!(reason.contains("differs"), "{reason}");
}

// --- The guards and the whole-guide run (`docs/GUIDE_FORMAT.md` §§7–9, 11) -------------------------------------

/// A repository in a scratch folder, holding `docs/guide/features.toml` (merging nothing but `memory-policies`,
/// which is flagged unmerged and has no presence probe, so the presence check is inconclusive for it and never
/// fails) and the given files, named relative to the repository. Since WE-a every probed feature is in the build,
/// so none can stand for an unmerged one in a whole-guide run.
fn mock_repo(label: &str, files: &[(&str, &str)]) -> Scratch {
    let repo = mineworld_test_support::scratch!(empty format!("{label}-repo"));
    let defaults = [(
        "docs/guide/features.toml",
        "[features]\nmemory-policies = false\n",
    )];
    for (path, text) in defaults.iter().chain(files) {
        files::write(&repo, path, text).expect("the repository file is written");
    }
    repo
}

/// Runs the whole guide of `repo` (`docs/guide`) against the real binary.
fn run_repo(label: &str, repo: &Scratch) -> Summary {
    run_guide::run(label, &repo.join("docs/guide"), repo)
}

/// Every failed check, as one string: its name, then its reason.
fn failures(summary: &Summary) -> String {
    summary
        .checks
        .iter()
        .filter_map(|(name, verdict)| match verdict {
            Verdict::Fail(reason) => Some(format!("{name}: {reason}")),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A page and its mirror, identical, under `docs/guide`.
fn page_and_mirror(name: &str, text: &str) -> Vec<(String, String)> {
    vec![
        (format!("docs/guide/{name}.md"), text.to_owned()),
        (format!("docs/guide/{name}.zh-CN.md"), text.to_owned()),
    ]
}

/// Builds a mock repository from owned (path, text) pairs, and returns it.
fn repo_with(label: &str, files: Vec<(String, String)>) -> Scratch {
    let borrowed: Vec<(&str, &str)> = files
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    mock_repo(label, &borrowed)
}

#[test]
fn a_correct_page_with_its_mirror_passes_every_guard() {
    let text = "### Step 1 — Check\n\nA sentence.\n\n```text\nshown only\n```\n\n```console\n$ mineworld --version\nmineworld [..]\n```\n";
    let repo = repo_with("guide-clean", page_and_mirror("a", text));
    let summary = run_repo("guide-clean-run", &repo);
    assert!(summary.no_failures(), "{}", summary.render());
}

#[test]
fn a_wrong_expected_output_turns_the_run_red_and_names_the_page_and_step() {
    let text = "### Step 2 — Check\n\n```console exit=1\n$ mineworld validate missing\nnot what it prints\n```\n";
    let repo = repo_with("guide-wrong", page_and_mirror("a", text));
    let summary = run_repo("guide-wrong-run", &repo);
    let found = failures(&summary);
    assert!(
        found.contains("page a line 3 step Step 2 — Check"),
        "{}",
        summary.render()
    );
    assert!(found.contains("printed other output"), "{found}");
}

#[test]
fn a_command_that_is_not_mineworld_turns_the_run_red() {
    let text = "```console\n$ cargo run\nx\n```\n";
    let repo = repo_with("guide-cargo", page_and_mirror("a", text));
    let found = failures(&run_repo("guide-cargo-run", &repo));
    assert!(found.contains("does not start with mineworld"), "{found}");
}

#[test]
fn an_unknown_directive_turns_the_run_red_and_names_the_page() {
    let text = "<!-- guide: teleport somewhere -->\n";
    let repo = repo_with("guide-directive", page_and_mirror("a", text));
    let found = failures(&run_repo("guide-directive-run", &repo));
    assert!(found.contains("page a"), "{found}");
    assert!(found.contains("teleport"), "{found}");
}

#[test]
fn a_file_block_path_outside_the_scratch_folder_turns_the_run_red() {
    let text = "```yaml file=../x\nx: 1\n```\n";
    let repo = repo_with("guide-escape", page_and_mirror("a", text));
    let found = failures(&run_repo("guide-escape-run", &repo));
    assert!(found.contains("refused"), "{found}");
}

#[test]
fn an_equals_mismatch_turns_the_run_red_and_names_the_folder() {
    let text =
        "<!-- guide: equals w docs/guide/examples/w -->\n\n```yaml file=w/x.yaml\na: 2\n```\n";
    let repo = repo_with(
        "guide-equals-red",
        page_and_mirror("a", text)
            .into_iter()
            .chain([(
                "docs/guide/examples/w/x.yaml".to_owned(),
                "a: 1\n".to_owned(),
            )])
            .collect(),
    );
    let found = failures(&run_repo("guide-equals-red-run", &repo));
    assert!(found.contains("equals w docs/guide/examples/w"), "{found}");
    assert!(found.contains("differs"), "{found}");
}

#[test]
fn a_mirror_whose_code_block_differs_by_one_byte_turns_the_run_red() {
    let english = "```text\nshown\n```\n";
    let mirror = "```text\nshowN\n```\n";
    let repo = repo_with(
        "guide-mirror",
        vec![
            ("docs/guide/a.md".to_owned(), english.to_owned()),
            ("docs/guide/a.zh-CN.md".to_owned(), mirror.to_owned()),
        ],
    );
    let found = failures(&run_repo("guide-mirror-run", &repo));
    assert!(found.contains("mirror of a"), "{found}");
    assert!(found.contains("code block 1 differs"), "{found}");
}

#[test]
fn a_mirror_missing_its_english_page_or_its_mirror_turns_the_run_red() {
    let orphan = repo_with(
        "guide-orphan",
        vec![("docs/guide/b.zh-CN.md".to_owned(), "x\n".to_owned())],
    );
    assert!(failures(&run_repo("guide-orphan-run", &orphan)).contains("b.zh-CN.md"));

    let unmirrored = repo_with(
        "guide-unmirrored",
        vec![("docs/guide/c.md".to_owned(), "x\n".to_owned())],
    );
    assert!(failures(&run_repo("guide-unmirrored-run", &unmirrored)).contains("no mirror"));
}

#[test]
fn the_index_and_its_mirror_are_neither_run_nor_held_to_page_parity() {
    // A block that would fail if it ran, and a mirror whose English page is the index: both are the
    // index's, which is read by people and never executed (§1).
    let index = "```console\n$ ls\n```\n";
    let repo = repo_with(
        "guide-index",
        vec![
            ("docs/guide/README.md".to_owned(), index.to_owned()),
            ("docs/guide/README.zh-CN.md".to_owned(), "索引\n".to_owned()),
        ],
    );
    let summary = run_repo("guide-index-run", &repo);
    assert!(summary.no_failures(), "{}", summary.render());
    assert_eq!(summary.passed, 0, "{}", summary.render());
}

#[test]
fn a_link_to_a_missing_reference_anchor_turns_the_run_red() {
    let text = "[corner](../reference/index.md#missing)\n";
    let repo = repo_with(
        "guide-anchor",
        page_and_mirror("a", text)
            .into_iter()
            .chain([(
                "docs/reference/index.md".to_owned(),
                "## Corner\n".to_owned(),
            )])
            .collect(),
    );
    let found = failures(&run_repo("guide-anchor-run", &repo));
    assert!(found.contains("no heading with that anchor"), "{found}");
}

#[test]
fn a_broken_link_in_a_mirror_turns_the_run_red_and_names_the_mirror() {
    // §7 rule 4: a mirror's prose may differ, so its links are its own, and each must resolve.
    let repo = repo_with(
        "guide-mirror-link",
        vec![
            ("docs/guide/a.md".to_owned(), "[b](b.md)\n".to_owned()),
            (
                "docs/guide/a.zh-CN.md".to_owned(),
                "[b](b.zh-CN.md#无)\n".to_owned(),
            ),
            ("docs/guide/b.md".to_owned(), "## B\n".to_owned()),
            ("docs/guide/b.zh-CN.md".to_owned(), "## 乙\n".to_owned()),
        ],
    );
    let found = failures(&run_repo("guide-mirror-link-run", &repo));
    assert!(found.contains("a.zh-CN.md: b.zh-CN.md#无"), "{found}");
}

#[test]
fn a_link_into_a_reference_that_is_not_yet_generated_is_inconclusive_not_passed() {
    let text = "[corner](../reference/index.md#corner)\n";
    let repo = repo_with("guide-unbuilt", page_and_mirror("a", text));
    let summary = run_repo("guide-unbuilt-run", &repo);
    assert!(summary.no_failures(), "{}", summary.render());
    assert!(
        summary
            .checks
            .iter()
            .any(|(name, verdict)| name == "reference links"
                && matches!(verdict, Verdict::Inconclusive(_))),
        "{}",
        summary.render()
    );
}

#[test]
fn a_feature_flagged_false_that_the_build_has_turns_the_run_red() {
    let repo = mock_repo(
        "guide-flag-false",
        &[("docs/guide/features.toml", "[features]\ncalendar = false\n")],
    );
    let found = failures(&run_repo("guide-flag-false-run", &repo));
    assert!(found.contains("feature calendar"), "{found}");
    assert!(found.contains("is in the build"), "{found}");
}

/// Every probed feature is in this build since WE-a, so the "flagged true, not built" direction is checked on
/// the presence check itself, over a build whose `mineworld packs list` lacks the probe's pack.
#[test]
fn a_feature_flagged_true_that_the_build_lacks_turns_the_run_red() {
    let flags =
        features::read("[features]\ncalendar = true\nhappenings = true\n").expect("features");
    let build = features::Build {
        packs: Ok(
            "system-pack  mineworld-calendar  0.1.0\nsystem-pack  mineworld-weather  0.1.0\n"
                .to_owned(),
        ),
        models: Ok(Vec::new()),
        reference: Ok(std::collections::BTreeMap::new()),
    };
    let verdicts = features::against_build(&flags, &build);
    assert_eq!(
        verdicts[0],
        ("calendar".to_owned(), Verdict::Pass),
        "{verdicts:?}"
    );
    assert!(
        matches!(&verdicts[1], (name, Verdict::Fail(reason))
            if name == "happenings" && reason.contains("happenings = true")),
        "{verdicts:?}"
    );
}

#[test]
fn a_block_for_an_unmerged_feature_is_listed_as_pending_and_never_as_passed() {
    let text = "### Step 1 — Wait\n\n<!-- guide: requires memory-policies -->\n\n```console\n$ mineworld packs list\n```\n";
    let repo = repo_with("guide-pending", page_and_mirror("a", text));
    let summary = run_repo("guide-pending-run", &repo);
    assert!(summary.no_failures(), "{}", summary.render());
    assert_eq!(summary.passed, 0, "a pending block is not a pass");
    assert!(
        summary
            .pending
            .iter()
            .any(|item| item.contains("step Step 1 — Wait") && item.contains("memory-policies")),
        "{}",
        summary.render()
    );
}

#[test]
fn the_guide_in_this_repository_runs_green() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let summary = run_guide::run("repo-guide", &repo.join("docs/guide"), &repo);
    // Written past the test harness's capture, so every CI log shows the counts and every block that did not
    // run, by name, even when the test passes (§11).
    let _ = std::io::Write::write_all(
        &mut std::io::stderr(),
        format!("{}\n", summary.render()).as_bytes(),
    );
    assert!(summary.no_failures(), "{}", summary.render());
    // §11.1: no block stops running without a reviewed change. The blocks that do not run are exactly the
    // committed list, and at least the committed number of blocks ran and passed.
    let expected = expected_run();
    let mut not_run = summary.not_run.clone();
    not_run.sort();
    assert_eq!(
        not_run,
        expected.not_run,
        "the blocks that did not run are not the committed list (tools/cli/tests/guide/expected_run.toml):\n{}",
        summary.render()
    );
    assert!(
        summary.passed >= expected.minimum_passed,
        "{} block(s) passed, fewer than the committed minimum of {}:\n{}",
        summary.passed,
        expected.minimum_passed,
        summary.render()
    );
}

/// The real guide's committed run (§11.1): the blocks that may not run, and the fewest that must pass.
struct ExpectedRun {
    not_run: Vec<String>,
    minimum_passed: usize,
}

fn expected_run() -> ExpectedRun {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/guide/expected_run.toml"
    ))
    .expect("expected_run.toml is read");
    let table: toml::Table = text.parse().expect("expected_run.toml is TOML");
    let minimum_passed = table
        .get("minimum_passed")
        .and_then(toml::Value::as_integer)
        .and_then(|count| usize::try_from(count).ok())
        .expect("minimum_passed is a count");
    let mut not_run: Vec<String> = table
        .get("not_run")
        .and_then(toml::Value::as_array)
        .expect("not_run is a list")
        .iter()
        .map(|item| item.as_str().expect("each entry is text").to_owned())
        .collect();
    not_run.sort();
    ExpectedRun {
        not_run,
        minimum_passed,
    }
}

// --- PR 25e: the comparison folder and the probes beyond packs (`docs/GUIDE_FORMAT.md` §§1.1, 9.1)

#[test]
fn comparison_pages_run_after_the_chapters_and_are_held_to_their_mirrors() {
    // A comparison page that continues a chapter sees the chapter's folder: it runs after every chapter.
    let chapter =
        "```yaml file=w/x.yaml\nk: 1\n```\n\n```console exit=1\n$ mineworld validate w\n...\n```\n";
    let compare = "<!-- guide: continues 01-a -->\n\n```sh\ncp -R w v\n```\n\n```powershell\nCopy-Item -Recurse w v\n```\n\n```console exit=1\n$ mineworld validate v\n...\n```\n";
    let mut files = page_and_mirror("01-a", chapter);
    files.extend(page_and_mirror("compare/b", compare));
    files.push((
        "docs/guide/compare/README.md".to_owned(),
        "```console\n$ ls\n```\n".to_owned(),
    ));
    let repo = repo_with("guide-compare", files);
    let summary = run_repo("guide-compare-run", &repo);
    assert!(summary.no_failures(), "{}", summary.render());
    assert_eq!(summary.passed, 4, "{}", summary.render());
    assert!(
        summary
            .checks
            .iter()
            .any(|(name, verdict)| name == "mirror of compare/b" && *verdict == Verdict::Pass),
        "{}",
        summary.render()
    );

    let lonely = repo_with(
        "guide-compare-lonely",
        vec![("docs/guide/compare/c.md".to_owned(), "Text.\n".to_owned())],
    );
    let found = failures(&run_repo("guide-compare-lonely-run", &lonely));
    assert!(
        found.contains("compare/c.md has no mirror compare/c.zh-CN.md"),
        "{found}"
    );
}

#[test]
fn model_and_reference_probes_turn_a_wrong_flag_red() {
    let repo = mock_repo(
        "guide-probes",
        &[(
            "docs/guide/features.toml",
            "[features]\nmotivated = false\nrelationship-decay = false\nlm-controller = false\nhappenings = false\n",
        )],
    );
    let found = failures(&run_repo("guide-probes-run", &repo));
    assert!(
        found.contains("motivated = false, but decision model motivated is in the build"),
        "{found}"
    );
    assert!(
        found.contains("relationship-decay = false, but \"`familiarity_decay`\""),
        "{found}"
    );
    assert!(
        found.contains("lm-controller = false, but decision model lm is in the build"),
        "{found}"
    );
    assert!(
        found.contains("happenings = false, but pack mineworld-happenings is in the build"),
        "{found}"
    );
}
