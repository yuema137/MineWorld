//! S30-k1 C4 and AC-k-3 / AC-k-4 (`pr-s30k-localisation.md` §§5.8, 5.10): `attributes check --locale`
//! and `attributes show --locale` through the real binary, on both fixture worlds. Every expectation is
//! hand-written from the fixture label files, never computed with the code under test.

mod headless;

use std::path::Path;

use headless::{Tables, fresh, mineworld, stderr, stdout};

const CONSUMERS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/attributes-consumers"
);
const ELSEWHERE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/labels-elsewhere"
);
const ENTITIES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../entities");

/// Runs `mineworld attributes <arguments> --packs <entities>`, which must exit 0; its stdout.
fn attributes(arguments: &[&str]) -> String {
    let mut all = vec!["attributes"];
    all.extend_from_slice(arguments);
    all.extend_from_slice(&["--packs", ENTITIES]);
    let output = mineworld(&all);
    assert!(output.status.success(), "{}", stderr(&output));
    stdout(&output)
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("the world lists") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("a copy");
        }
    }
}

/// The `missing <locale> …` lines of a `check` print, in order.
fn missing<'a>(printed: &'a str, locale: &str) -> Vec<&'a str> {
    let prefix = format!("missing {locale} ");
    printed
        .lines()
        .filter(|line| line.starts_with(&prefix))
        .collect()
}

#[test]
fn check_lists_the_deliberate_gap_of_world_one_and_stays_clean() {
    let printed = attributes(&["check", CONSUMERS, "--locale", "zh_Hans"]);
    let gaps = missing(&printed, "zh_Hans");
    assert!(
        gaps.iter()
            .any(|line| line.starts_with("missing zh_Hans core.hair-colour words.grey")),
        "the missing grey is listed: {printed}"
    );
    assert!(
        !gaps
            .iter()
            .any(|line| line.contains("core.glasses")
                || line.contains("attributes-consumers.freckles")),
        "a complete entry is never listed: {printed}"
    );
    assert!(
        printed.lines().any(|line| line.starts_with("zh_Hans: ")
            && line.ends_with(&format!("{} missing piece(s)", gaps.len()))),
        "one summary line counts the listed pieces: {printed}"
    );
    assert!(printed.contains("attribute(s); clean"), "{printed}");
}

#[test]
fn check_reads_zh_hyphen_hans_as_zh_hans_and_names_an_unoffered_locale() {
    let typed = attributes(&["check", CONSUMERS, "--locale", "zh-Hans"]);
    let canonical = attributes(&["check", CONSUMERS, "--locale", "zh_Hans"]);
    assert_eq!(typed, canonical, "zh-Hans is zh_Hans (§5.1)");
    let printed = attributes(&["check", CONSUMERS, "--locale", "de"]);
    assert!(
        printed.lines().any(|line| line == "de: not offered"),
        "{printed}"
    );
    assert!(missing(&printed, "de").is_empty(), "{printed}");
}

#[test]
fn check_without_a_locale_prints_no_locale_line() {
    let printed = attributes(&["check", CONSUMERS]);
    assert!(
        !printed.lines().any(|line| line.starts_with("missing ")
            || line.starts_with("zh_Hans:")
            || line.starts_with("fr:")),
        "{printed}"
    );
}

#[test]
fn world_two_translates_its_own_attribute_in_both_locales() {
    let printed = attributes(&["check", ELSEWHERE, "--locale", "zh_Hans", "--locale", "fr"]);
    for locale in ["zh_Hans", "fr"] {
        assert!(
            !missing(&printed, locale)
                .iter()
                .any(|line| line.contains("labels-elsewhere.sea-legs")),
            "{locale}: sea-legs is complete: {printed}"
        );
    }
    let chinese = attributes(&["show", ELSEWHERE, "--person", "ines", "--locale", "zh_Hans"]);
    assert!(chinese.contains("赤褐色头发"), "{chinese}");
    assert!(chinese.contains("在甲板上稳如老水手"), "{chinese}");
    let french = attributes(&["show", ELSEWHERE, "--person", "tomas", "--locale", "fr"]);
    assert!(french.contains("cheveux noirs"), "{french}");
    assert!(french.contains("chancelant sur le pont"), "{french}");
    let english = attributes(&["show", ELSEWHERE, "--person", "ines"]);
    assert!(english.contains("sure-footed on deck"), "{english}");
    assert!(!english.contains("(en)"), "{english}");
}

#[test]
fn removing_a_label_file_is_data_only_and_falls_back_to_english() {
    let scratch = fresh("labels-elsewhere-without-fr");
    let world = scratch.path().join("labels-elsewhere");
    copy_dir(Path::new(ELSEWHERE), &world);
    std::fs::remove_file(world.join("labels/fr.yaml")).expect("the fr file is removed");
    let world = world.to_str().expect("UTF-8");
    let printed = attributes(&["check", world, "--locale", "fr"]);
    assert!(
        printed.lines().any(|line| line == "fr: not offered"),
        "{printed}"
    );
    let shown = attributes(&["show", world, "--person", "ines", "--locale", "fr"]);
    assert!(
        shown
            .lines()
            .next()
            .is_some_and(|line| line == "fr: not offered; shown in English"),
        "{shown}"
    );
    assert!(shown.contains("sure-footed on deck"), "{shown}");
    assert!(!shown.contains("(en)"), "{shown}");
}

#[test]
fn show_marks_every_english_fallback_and_never_widens_the_observer() {
    let shown = attributes(&[
        "show", CONSUMERS, "--person", "alice", "--locale", "zh_Hans",
    ]);
    let other = shown
        .lines()
        .find(|line| line.trim_start().starts_with("another observer:"))
        .expect("the observer line");
    assert!(
        other.contains("grey hair (en)"),
        "the gap is marked: {other}"
    );
    assert!(other.contains("六十多岁"), "{other}");
    assert!(
        !other.contains("秘密") && !other.contains("secret"),
        "nothing private reaches another observer: {other}"
    );
}

/// `run` on `world` to `days`, kept in `save` (resumed when it exists), which must succeed.
fn run(world: &str, days: &str, save: &Path) {
    let save = save.to_str().expect("UTF-8");
    let output = mineworld(&[
        "run",
        world,
        "--headless",
        "--seed",
        "3",
        "--days",
        days,
        "--save",
        save,
        "--packs",
        ENTITIES,
    ]);
    assert!(
        output.status.success(),
        "{}\n{}",
        stderr(&output),
        stdout(&output)
    );
}

/// AC-k-6: the same world with and without all its label files states byte-identical facts, journal and
/// sealed blocks; a save resumed after a label file edit runs on without drift.
#[test]
fn labels_never_reach_a_fact_and_a_label_edit_resumes_without_drift() {
    let scratch = fresh("labels-elsewhere-digests");
    let labelled = scratch.path().join("labelled/labels-elsewhere");
    let bare = scratch.path().join("bare/labels-elsewhere");
    copy_dir(Path::new(ELSEWHERE), &labelled);
    copy_dir(Path::new(ELSEWHERE), &bare);
    std::fs::remove_dir_all(bare.join("labels")).expect("the labels are removed");
    let (labelled_world, bare_world) = (
        labelled.to_str().expect("UTF-8"),
        bare.to_str().expect("UTF-8"),
    );
    let (labelled_save, bare_save) = (scratch.path().join("save-a"), scratch.path().join("save-b"));
    run(labelled_world, "1", &labelled_save);
    run(bare_world, "1", &bare_save);
    let same = |a: &Tables, b: &Tables, when: &str| {
        assert!(!a.facts.is_empty(), "{when}: the world stated facts");
        assert!(a.facts == b.facts, "{when}: facts differ");
        assert!(a.journal == b.journal, "{when}: journal differs");
        assert!(a.sealed == b.sealed, "{when}: sealed blocks differ");
    };
    same(
        &Tables::read(&labelled_save),
        &Tables::read(&bare_save),
        "day 1",
    );

    let file = labelled.join("labels/zh_Hans.yaml");
    let edited = std::fs::read_to_string(&file)
        .expect("the label file reads")
        .replace("稳如老水手", "像老水手一样稳");
    std::fs::write(&file, edited).expect("the label file is edited");
    run(labelled_world, "2", &labelled_save);
    run(bare_world, "2", &bare_save);
    same(
        &Tables::read(&labelled_save),
        &Tables::read(&bare_save),
        "day 2, resumed",
    );
    let shown = attributes(&[
        "show",
        labelled_world,
        "--person",
        "ines",
        "--locale",
        "zh_Hans",
    ]);
    assert!(
        shown.contains("在甲板上像老水手一样稳"),
        "the new words are served: {shown}"
    );
}
