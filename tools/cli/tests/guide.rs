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
#[path = "guide/page.rs"]
mod page;
#[path = "guide/page_run.rs"]
mod page_run;

use features::Features;
use page::{Body, Directive, parse};
use page_run::{Outcome, run_page};

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

    let copy_after_waiting = "<!-- guide: requires calendar -->\n\n<!-- guide: copy a b -->\n";
    assert!(parse(copy_after_waiting).is_err());
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
    let (reports, _) = run_page("fixture", &fixture(), &scratch, &Features::new(), None);
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
    let scratch = mineworld_test_support::scratch!(empty "guide-wrong-output");
    let (reports, _) = run_page(
        "wrong",
        &parse(page).expect("parses"),
        &scratch,
        &Features::new(),
        None,
    );
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
    let (reports, _) = run_page(
        "piped",
        &parse(piped).expect("parses"),
        &scratch,
        &Features::new(),
        None,
    );
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
    let (reports, _) = run_page(
        "exit",
        &parse(wrong).expect("parses"),
        &scratch,
        &Features::new(),
        None,
    );
    let Outcome::Failed(reason) = &reports[0].outcome else {
        panic!("exit 1 against an expected 0 fails: {reports:#?}");
    };
    assert!(reason.contains("exited with 1"), "{reason}");

    let expected = "```console exit=1\n$ mineworld validate nothing-here\n[mineworld] nothing-here is not a World Pack directory\n```\n";
    let (reports, _) = run_page(
        "exit",
        &parse(expected).expect("parses"),
        &scratch,
        &Features::new(),
        None,
    );
    assert_eq!(reports[0].outcome, Outcome::Passed, "{reports:#?}");
}

#[test]
fn a_block_waiting_for_an_unmerged_feature_is_pending_and_so_is_the_rest_of_its_page() {
    let page = "### Step 1 — Wait\n\n<!-- guide: requires happenings -->\n\n```console\n$ mineworld validate x\n```\n\n\
### Step 2 — After\n\n```console\n$ mineworld validate x\n```\n";
    let scratch = mineworld_test_support::scratch!(empty "guide-pending");
    let features = features::read("[features]\nhappenings = false\n").expect("features");
    let (reports, carried) = run_page(
        "waiting",
        &parse(page).expect("parses"),
        &scratch,
        &features,
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
    let (reports, _) = run_page(
        "unknown",
        &parse(page).expect("parses"),
        &scratch,
        &Features::new(),
        None,
    );
    let Outcome::Failed(reason) = &reports[0].outcome else {
        panic!("an unknown feature fails: {reports:#?}");
    };
    assert!(reason.contains("teleporting"), "{reason}");
}

#[test]
fn a_file_block_then_copy_leaves_the_folder_under_its_new_name_in_the_scratch_folder() {
    let page = "```yaml file=a/x.yaml\nk: 1\n```\n\n<!-- guide: copy a b -->\n";
    let scratch = mineworld_test_support::scratch!(empty "guide-copy");
    let (reports, _) = run_page(
        "copy",
        &parse(page).expect("parses"),
        &scratch,
        &Features::new(),
        None,
    );
    assert!(
        reports
            .iter()
            .all(|report| report.outcome == Outcome::Passed),
        "{reports:#?}"
    );
    assert_eq!(
        std::fs::read_to_string(scratch.join("b/x.yaml")).expect("copied"),
        "k: 1\n"
    );
}
