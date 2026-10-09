//! `mineworld packs` through the real binary: every pack this build and the named directories provide
//! has a package identity, printed; every bad one is refused by name with a non-zero exit
//! (`docs/DECISIONS.md` `ARC-53`; step-16 §14.4 EA-1, EA-3, EA-4, EA-5).
//!
//! The installed set is the oracle for which System Packs exist (`AVAILABLE`), never a list written
//! here: a pack is located by its system id, then the packs are counted (`ARC-23`). No crate of a
//! market pack is named in this file (`ARC-35` check 2).

use std::path::Path;
use std::process::Command;

use mineworld_packages::License;
use mineworld_worldpack::catalog::AVAILABLE;

const REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// `mineworld <arguments>`: (success, stdout, stderr).
fn mineworld(arguments: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_mineworld"))
        .args(arguments)
        .output()
        .expect("the mineworld binary runs");
    (
        output.status.success(),
        String::from_utf8(output.stdout).expect("UTF-8"),
        String::from_utf8(output.stderr).expect("UTF-8"),
    )
}

fn repository(relative: &str) -> String {
    Path::new(REPOSITORY)
        .join(relative)
        .to_str()
        .expect("a UTF-8 path")
        .to_owned()
}

/// A directory of its own, removed when the test ends (scratch, DEP-29).
struct Scratch(mineworld_test_support::Scratch);

impl Scratch {
    fn new(name: &str) -> Self {
        Self(mineworld_test_support::scratch!(empty format!("packs-{name}")))
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("parent directory");
        std::fs::write(path, text).expect("file writes");
    }

    fn path(&self) -> &str {
        self.0.to_str().expect("a UTF-8 path")
    }
}

/// One line of `packs list`: type, id, version, licence, the rest.
struct Line<'a> {
    kind: &'a str,
    id: &'a str,
    version: &'a str,
    license: &'a str,
    rest: Vec<&'a str>,
}

fn lines(listing: &str) -> Vec<Line<'_>> {
    listing
        .lines()
        .filter(|line| !line.ends_with(" packs"))
        .map(|line| {
            let mut words = line.split_whitespace();
            let mut next = || words.next().expect("a listed field");
            let (kind, id, version, license) = (next(), next(), next(), next());
            Line {
                kind,
                id,
                version,
                license,
                rest: words.collect(),
            }
        })
        .collect()
}

/// EA-1 and EA-3: every System Pack of the installed set, then the controller, each with a version
/// that is the framework's, an SPDX licence and an author.
#[test]
fn every_code_pack_in_the_build_is_listed_with_its_identity() {
    let (ok, listing, stderr) = mineworld(&["packs", "list"]);
    assert!(ok, "packs list failed: {stderr}");
    let listed = lines(&listing);

    let systems: Vec<&Line<'_>> = listed.iter().filter(|l| l.kind == "system-pack").collect();
    let system_id = |line: &Line<'_>| match line.rest.as_slice() {
        [.., "system", id] => (*id).to_owned(),
        other => panic!("{}: no system id in {other:?}", line.id),
    };
    let presence = systems
        .iter()
        .find(|line| system_id(line) == "presence")
        .expect("presence is located first");
    assert_eq!(presence.id, "mineworld-presence");

    let printed: Vec<String> = systems.iter().map(|line| system_id(line)).collect();
    let installed: Vec<String> = AVAILABLE.iter().map(|c| c.id().to_string()).collect();
    assert_eq!(
        printed, installed,
        "one line per installed pack, in its order"
    );

    let controllers: Vec<&str> = listed
        .iter()
        .filter(|l| l.kind == "controller-pack")
        .map(|l| l.id)
        .collect();
    assert_eq!(controllers, ["mineworld-rule-controller"]);

    for line in listed.iter().filter(|l| l.kind != "world-pack") {
        assert_eq!(
            line.version,
            env!("CARGO_PKG_VERSION"),
            "{} is not at the framework's version",
            line.id
        );
        License::new(line.license).unwrap_or_else(|error| panic!("{}: {error}", line.id));
        assert!(
            line.rest.first().is_some_and(|author| *author != "—"),
            "{}: no author",
            line.id
        );
    }
    assert!(
        listing.ends_with(&format!("{} packs\n", AVAILABLE.len() + 1)),
        "{listing}"
    );
}

/// EC-6 (`ARC-66` point 4): every listed System Pack says, just before its system id, whether it was
/// compiled from this workspace — `bundled` exactly when the installed set says so, `third-party`
/// otherwise. The oracle is each capability's own `package().bundled()`, never a list written here.
#[test]
fn every_system_pack_is_listed_bundled_or_third_party_as_the_build_compiled_it() {
    let (ok, listing, stderr) = mineworld(&["packs", "list"]);
    assert!(ok, "packs list failed: {stderr}");
    let listed = lines(&listing);
    let origins: Vec<(String, String)> = listed
        .iter()
        .filter(|l| l.kind == "system-pack")
        .map(|line| match line.rest.as_slice() {
            [.., origin, "system", id] => ((*id).to_owned(), (*origin).to_owned()),
            other => panic!("{}: no origin and system id in {other:?}", line.id),
        })
        .collect();
    let expected: Vec<(String, String)> = AVAILABLE
        .iter()
        .map(|c| {
            let word = if c.package().bundled() {
                "bundled"
            } else {
                "third-party"
            };
            (c.id().to_string(), word.to_owned())
        })
        .collect();
    assert_eq!(origins, expected);
    let bundled = origins.iter().filter(|(_, o)| o == "bundled").count();
    println!("{bundled} bundled, {} third-party", origins.len() - bundled);
    assert!(
        bundled > 0,
        "the framework's own packs are located as bundled"
    );
}

/// EA-4 and EA-5: the data packs in the named directories — every world under worlds/ and the two
/// default presentation packs; LICENSES/ beside them is not a pack — and each validates.
#[test]
fn the_data_packs_in_named_directories_are_listed_and_validate() {
    let worlds = repository("worlds");
    let presentation = repository("presentation/mineworld-default");
    let (ok, listing, stderr) = mineworld(&[
        "packs",
        "list",
        "--packs",
        &worlds,
        "--packs",
        &presentation,
    ]);
    assert!(ok, "packs list failed: {stderr}");
    let listed = lines(&listing);

    let world_directories: Vec<String> = std::fs::read_dir(&worlds)
        .expect("worlds/ lists")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.join("world.yaml").is_file())
        .map(|path| {
            path.file_name()
                .expect("a name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    let mut listed_worlds: Vec<&str> = listed
        .iter()
        .filter(|l| l.kind == "world-pack")
        .map(|l| l.id)
        .collect();
    let mut expected: Vec<&str> = world_directories.iter().map(String::as_str).collect();
    listed_worlds.sort_unstable();
    expected.sort_unstable();
    assert!(expected.contains(&"social-cafe"), "located first");
    assert_eq!(listed_worlds, expected);

    let presentation_packs: Vec<&str> = listed
        .iter()
        .filter(|l| l.kind == "presentation-pack")
        .map(|l| l.id)
        .collect();
    assert_eq!(
        presentation_packs,
        ["mineworld-default-2d", "mineworld-default-3d"]
    );
    assert!(!listing.contains("LICENSES"), "{listing}");

    for directory in world_directories
        .iter()
        .map(|name| format!("{worlds}/{name}"))
        .chain(["2D", "3D"].map(|d| format!("{presentation}/{d}")))
    {
        let (ok, out, stderr) = mineworld(&["packs", "validate", &directory]);
        assert!(ok, "{directory}: {stderr}");
        assert!(out.contains("is a valid"), "{out}");
    }
}

/// `packs show`: a System Pack's system id and contract version, located through the installed set;
/// an unknown id refused, listing the ids that exist.
#[test]
fn show_prints_one_identity_and_refuses_an_unknown_id() {
    let presence = AVAILABLE
        .into_iter()
        .find(|c| c.id().as_str() == "presence")
        .expect("presence is installed");
    let (ok, out, stderr) = mineworld(&["packs", "show", presence.package().name()]);
    assert!(ok, "{stderr}");
    assert!(
        out.contains(&format!(
            "system      presence (SystemVersion {})",
            presence.version().get()
        )),
        "{out}"
    );

    let (ok, _, stderr) = mineworld(&["packs", "show", "no-such-pack"]);
    assert!(!ok);
    assert!(
        stderr.contains("'no-such-pack'") && stderr.contains("mineworld-presence"),
        "{stderr}"
    );
}

const PACK_FILE: &str = "id: style-a\ntype: presentation-pack\nversion: 0.1.0\nmineworld: \"^0.1\"\n\
                         license: MIT\nauthors: [Somebody]\n";

/// EA-4 end to end: a bad pack in a named directory fails `packs list` with exit 1, naming it.
#[test]
fn a_bad_pack_in_a_named_directory_is_refused_by_name() {
    for (case, files, needles) in [
        (
            "semver",
            vec![("a/pack.yaml", PACK_FILE.replace("0.1.0", "0.1"))],
            &["pack.yaml", "'0.1' is not a semver version"][..],
        ),
        (
            "licence",
            vec![("a/pack.yaml", PACK_FILE.replace("MIT", "NOT-A-LICENCE"))],
            &[
                "pack.yaml",
                "'NOT-A-LICENCE' is not an SPDX licence expression",
            ][..],
        ),
        (
            "duplicate",
            vec![
                ("a/pack.yaml", PACK_FILE.to_owned()),
                ("b/pack.yaml", PACK_FILE.to_owned()),
            ],
            &[
                "two packs have the id style-a",
                "packs-duplicate/a",
                "packs-duplicate/b",
            ][..],
        ),
    ] {
        let scratch = Scratch::new(case);
        for (relative, text) in &files {
            scratch.write(relative, text);
        }
        let (ok, _, stderr) = mineworld(&["packs", "list", "--packs", scratch.path()]);
        assert!(!ok, "{case}: accepted");
        for needle in needles {
            // A path in a needle is written with `/`; the refusal prints this platform's separator
            // (step-16 §16.12 PD-p5; the claim is unchanged).
            let needle = needle.replace('/', std::path::MAIN_SEPARATOR_STR);
            assert!(
                stderr.contains(&needle),
                "{case}: {needle:?} not in {stderr}"
            );
        }
    }
}

/// EA-5 end to end: a world without its package fields still runs and validates as a world, and is
/// refused as a pack, naming the field.
#[test]
fn a_world_without_its_package_fields_is_a_world_but_not_a_valid_pack() {
    let scratch = Scratch::new("bare");
    scratch.write(
        "bare-world/world.yaml",
        "world:\n  id: bare-world\n  name: Bare\nsystems: [presence]\nplaces: [cafe]\n\
         population: [alice]\n",
    );
    scratch.write("bare-world/places/cafe.yaml", "tags: [cafe]\n");
    scratch.write(
        "bare-world/people/alice.yaml",
        "tags: [barista]\nlocation:\n  place: cafe\n",
    );
    let world = format!("{}/bare-world", scratch.path());

    let (ok, _, stderr) = mineworld(&["validate", &world]);
    assert!(ok, "the loader keeps the fields optional: {stderr}");
    let (ok, _, stderr) = mineworld(&["packs", "validate", &world]);
    assert!(!ok, "accepted without a version");
    assert!(
        stderr.contains("world.yaml") && stderr.contains("world.version"),
        "{stderr}"
    );
}
