//! Read step 4d with probe owners (no installed pack owns data yet): the order files are collected in
//! (I-b-7), the directory rule (F-S30b-3), and every refusal by name (I-b-9).

use std::collections::BTreeMap;

use mineworld_authoring::{DataRefusal, OwnsData};
use mineworld_kernel::SystemIdentity;
use serde::Deserialize;

use super::*;

const ONE: DataDirectory = DataDirectory::from_static("one");
const TWO: DataDirectory = DataDirectory::from_static("two");

/// The owner of `one/` and `two/`: each file a map of strings; its context the origins it was handed,
/// in order. A file stating `refuse: …` is refused as a whole, naming that file.
struct AlphaData;

impl SystemIdentity for AlphaData {
    const ID: SystemId = SystemId::from_static("alpha");
}

/// Where one file came from, as the owner saw it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Seen {
    pack: String,
    source: DataSource,
    path: String,
}

impl OwnsData for AlphaData {
    const DIRECTORIES: &'static [DataDirectory] = &[ONE, TWO];
    type File = BTreeMap<String, String>;
    type Context = Vec<Seen>;

    fn decode_file<'de, D: Deserializer<'de>>(
        _: DataDirectory,
        deserializer: D,
    ) -> Result<Self::File, D::Error> {
        BTreeMap::deserialize(deserializer)
    }

    fn context(
        files: Vec<(DataOrigin, Self::File)>,
        _: Option<&dyn Any>,
    ) -> Result<Vec<Seen>, DataRefusal> {
        let mut seen = Vec::new();
        for (origin, file) in files {
            if let Some(why) = file.get("refuse") {
                return Err(DataRefusal {
                    file: Some(origin.path),
                    reason: why.clone(),
                });
            }
            seen.push(Seen {
                pack: origin.pack,
                source: origin.source,
                path: origin.path.to_string_lossy().replace('\\', "/"),
            });
        }
        Ok(seen)
    }
}

/// Three probe owners: `Alpha` owns `one/` and `two/` and ships `one/self.yaml`; `Beta` owns nothing and
/// ships three files into Alpha's directories; `Gamma` owns nothing and ships into a directory nobody
/// owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Probe {
    Alpha,
    Beta,
    Gamma,
}

const ALPHA_SHIPS: &[ShippedData] = &[ShippedData::new("one/self.yaml", "from: alpha\n")];
const BETA_SHIPS: &[ShippedData] = &[
    ShippedData::new("two/b.yaml", "from: beta\n"),
    ShippedData::new("one/z.yaml", "from: beta\n"),
    ShippedData::new("one/a.yaml", "from: beta\n"),
];
const GAMMA_SHIPS: &[ShippedData] = &[ShippedData::new("nowhere/x.yaml", "from: gamma\n")];

impl DataOwnerOps for Probe {
    fn id(self) -> SystemId {
        match self {
            Self::Alpha => AlphaData::ID,
            Self::Beta => SystemId::from_static("beta"),
            Self::Gamma => SystemId::from_static("gamma"),
        }
    }

    fn directories(self) -> &'static [DataDirectory] {
        match self {
            Self::Alpha => AlphaData::DIRECTORIES,
            Self::Beta | Self::Gamma => &[],
        }
    }

    fn decode_file<'de, D: Deserializer<'de>>(
        self,
        directory: DataDirectory,
        deserializer: D,
    ) -> Result<Box<dyn Any + Send + Sync>, D::Error> {
        mineworld_authoring::decode_data_file::<AlphaData, D>(directory, deserializer)
    }

    fn context(
        self,
        files: Vec<(DataOrigin, Box<dyn Any + Send + Sync>)>,
        configuration: Option<&dyn AuthoredConfiguration>,
    ) -> Result<Arc<dyn OwnerContext>, DataRefusal> {
        mineworld_authoring::data_context::<AlphaData>(files, configuration)
    }

    fn ships(self) -> &'static [ShippedData] {
        match self {
            Self::Alpha => ALPHA_SHIPS,
            Self::Beta => BETA_SHIPS,
            Self::Gamma => GAMMA_SHIPS,
        }
    }
}

const INSTALLED: [Probe; 3] = [Probe::Alpha, Probe::Beta, Probe::Gamma];

/// A scratch directory under the system's temporary directory, removed on drop (as
/// `configure/tests.rs`' `ScratchRoot`).
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("mineworld-s30b-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a writable temporary directory");
        Self(path)
    }

    fn write(&self, relative: &str, text: &str) {
        let path = relative
            .split('/')
            .fold(self.0.clone(), |path, part| path.join(part));
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("writable");
        std::fs::write(path, text).expect("writable");
    }

    fn dir(&self, relative: &str) -> PathBuf {
        relative
            .split('/')
            .fold(self.0.clone(), |path, part| path.join(part))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn world<'a>(dir: &'a Path, admitted: &[&str]) -> DataRoot<'a> {
    DataRoot {
        pack: "a-world",
        source: DataSource::World,
        dir,
        format: &WORLD_DIRECTORIES,
        admitted: admitted.iter().map(|name| (*name).to_owned()).collect(),
    }
}

fn entity<'a>(pack: &'a str, dir: &'a Path) -> DataRoot<'a> {
    DataRoot {
        pack,
        source: DataSource::EntityPack,
        dir,
        format: &[mineworld_packages::ENTITY_ITEMS],
        admitted: Vec::new(),
    }
}

/// What Alpha's context saw, in order.
fn seen(contexts: &BTreeMap<Probe, Arc<dyn OwnerContext>>) -> Vec<(String, DataSource, String)> {
    contexts[&Probe::Alpha]
        .as_any()
        .downcast_ref::<Vec<Seen>>()
        .expect("alpha's context")
        .iter()
        .map(|seen| {
            let path = seen.path.rsplit('/').take(2).collect::<Vec<_>>();
            (
                seen.pack.clone(),
                seen.source,
                format!("{}/{}", path[1], path[0]),
            )
        })
        .collect()
}

/// I-b-7, AC-b-7's order half, AC-b-8 (M-b-3's test): shipped files first, in `systems:` order, each
/// pack's by directory then name; then each required Entity Pack in the order given (id order); then the
/// world — within each, the owner's directory order, then file name, whatever order the file system
/// lists them in.
#[test]
fn data_is_collected_in_composition_order() {
    let scratch = Scratch::new("order");
    scratch.write("world/two/c.yaml", "a: b\n");
    scratch.write("world/one/b.yaml", "a: b\n");
    scratch.write("world/one/a.yaml", "a: b\n");
    scratch.write("world/one/notes.txt", "not content\n");
    scratch.write("goods-a/one/x.yaml", "a: b\n");
    scratch.write("goods-b/two/y.yaml", "a: b\n");
    let (world_dir, a_dir, b_dir) = (
        scratch.dir("world"),
        scratch.dir("goods-a"),
        scratch.dir("goods-b"),
    );
    let roots = [
        entity("goods-a", &a_dir),
        entity("goods-b", &b_dir),
        world(&world_dir, &[]),
    ];

    // Beta is listed before Alpha in `systems:`, so Beta's shipped files come first.
    let contexts =
        contexts(&roots, &INSTALLED, &[Probe::Beta, Probe::Alpha], |_| None).expect("collects");
    use DataSource::{EntityPack, SystemPack, World};
    let expected: Vec<(String, DataSource, String)> = [
        ("beta", SystemPack, "one/a.yaml"),
        ("beta", SystemPack, "one/z.yaml"),
        ("beta", SystemPack, "two/b.yaml"),
        ("alpha", SystemPack, "one/self.yaml"),
        ("goods-a", EntityPack, "one/x.yaml"),
        ("goods-b", EntityPack, "two/y.yaml"),
        ("a-world", World, "one/a.yaml"),
        ("a-world", World, "one/b.yaml"),
        ("a-world", World, "two/c.yaml"),
    ]
    .into_iter()
    .map(|(pack, source, path)| (pack.to_owned(), source, path.to_owned()))
    .collect();
    assert_eq!(seen(&contexts), expected);
    assert_eq!(contexts.len(), 1, "only an owner of data gets a context");

    // The order is the name's, not the listing's: a listing in reverse comes back sorted.
    let listing: Vec<(OsString, PathBuf)> = ["c.yaml", "a.yaml", "b.yaml"]
        .into_iter()
        .map(|name| (OsString::from(name), PathBuf::from(name)))
        .collect();
    let names: Vec<OsString> = ordered(listing).into_iter().map(|(name, _)| name).collect();
    assert_eq!(names, ["a.yaml", "b.yaml", "c.yaml"]);
}

/// M-b-4's test: a directory whose name begins with `.` is not content — in a pack's root or inside a
/// data directory — and is neither refused nor read.
#[test]
fn a_dot_directory_is_not_content() {
    let scratch = Scratch::new("dot");
    scratch.write("world/.git/config", "[core]\n");
    scratch.write("world/.cache/one.yaml", "not: read\n");
    scratch.write("world/one/.hidden/inner.yaml", "not: read\n");
    scratch.write("world/one/a.yaml", "a: b\n");
    let dir = scratch.dir("world");
    let contexts = contexts(&[world(&dir, &[])], &INSTALLED, &[Probe::Alpha], |_| None)
        .expect("dot directories are ignored");
    assert_eq!(
        seen(&contexts),
        [
            (
                "alpha".to_owned(),
                DataSource::SystemPack,
                "one/self.yaml".to_owned()
            ),
            (
                "a-world".to_owned(),
                DataSource::World,
                "one/a.yaml".to_owned()
            )
        ]
    );
}

/// The directory rule (AC-b-3's in-crate half): an unknown directory is refused listing every directory
/// this build knows; `decisions:`'s directory is admitted; an installed owner's directory in a world that
/// does not enable it is refused naming the owner; a subdirectory inside a data directory is refused.
#[test]
fn the_directory_rule_admits_the_format_decisions_and_enabled_owners() {
    let scratch = Scratch::new("rule");
    scratch.write("world/people/ada.yaml", "tags: [x]\n");
    scratch.write("world/cognition/decisions.yaml", "x: y\n");
    let dir = scratch.dir("world");
    let admitted = world(&dir, &["cognition"]);
    check_directories(&admitted, &INSTALLED, &[Probe::Alpha]).expect("cognition is admitted");

    scratch.write("world/extras/notes.yaml", "x: y\n");
    let refusal = check_directories(&admitted, &INSTALLED, &[Probe::Alpha])
        .expect_err("extras is no directory of this pack");
    let PackError::UnknownDirectory { path, known } = &refusal else {
        panic!("got: {refusal}");
    };
    assert_eq!(*path, scratch.dir("world/extras"));
    assert_eq!(
        known,
        "cognition, configure, data, items, one, organizations, people, places, two"
    );
    std::fs::remove_dir_all(scratch.dir("world/extras")).expect("removed");

    scratch.write("world/one/a.yaml", "a: b\n");
    let refusal =
        check_directories(&admitted, &INSTALLED, &[Probe::Beta]).expect_err("alpha not enabled");
    let PackError::DataNeedsASystem {
        directory, system, ..
    } = &refusal
    else {
        panic!("got: {refusal}");
    };
    assert_eq!(directory, "one");
    assert_eq!(*system, AlphaData::ID);

    scratch.write("world/one/inner/b.yaml", "a: b\n");
    let refusal = contexts(&[admitted], &INSTALLED, &[Probe::Alpha], |_| None)
        .expect_err("a data directory holds files only");
    assert!(
        matches!(&refusal, PackError::UnknownDirectory { path, .. } if *path == scratch.dir("world/one/inner")),
        "got: {refusal}"
    );
}

/// A malformed file is refused at its line, naming it; a shipped one names its pack and path; the
/// owner's refusal of its data names the system and the file; a shipped file into a directory nobody
/// owns is refused naming the shipping pack.
#[test]
fn every_data_refusal_names_the_file_and_the_owner() {
    let scratch = Scratch::new("refusals");
    scratch.write("world/one/bad.yaml", "a: b\nc: [unclosed\n");
    let dir = scratch.dir("world");
    let refusal = contexts(&[world(&dir, &[])], &INSTALLED, &[Probe::Alpha], |_| None)
        .expect_err("malformed");
    let PackError::Malformed { path, kind, detail } = &refusal else {
        panic!("got: {refusal}");
    };
    assert_eq!(*path, scratch.dir("world/one/bad.yaml"));
    assert_eq!(*kind, "data");
    assert!(detail.contains("line"), "a position: {detail}");

    scratch.write("world/one/bad.yaml", "refuse: two files say so\n");
    let refusal = contexts(&[world(&dir, &[])], &INSTALLED, &[Probe::Alpha], |_| None)
        .expect_err("the owner refuses");
    let PackError::DataRefused {
        system,
        file,
        reason,
    } = &refusal
    else {
        panic!("got: {refusal}");
    };
    assert_eq!(*system, AlphaData::ID);
    assert_eq!(
        file.as_deref(),
        Some(scratch.dir("world/one/bad.yaml").as_path())
    );
    assert_eq!(reason, "two files say so");
    assert!(refusal.to_string().contains("bad.yaml"), "{refusal}");

    let empty = Scratch::new("refusals-empty");
    let refusal = contexts(
        &[world(&empty.0, &[])],
        &INSTALLED,
        &[Probe::Alpha, Probe::Gamma],
        |_| None,
    )
    .expect_err("nobody owns nowhere/");
    let PackError::UnknownDirectory { path, known } = &refusal else {
        panic!("got: {refusal}");
    };
    assert_eq!(
        *path,
        PathBuf::from("nowhere/x.yaml (shipped by the 'gamma' system)")
    );
    assert_eq!(known, "one, two");
}

/// No file is read for an owner the world does not enable — not even one shipped into its directory
/// (S30-a F-S30a-10 stays S30-c's to decide) — and an enabled owner with no files still gets a context.
#[test]
fn an_owner_not_enabled_reads_nothing_and_one_with_no_files_still_has_a_context() {
    let empty = Scratch::new("quiet");
    let contexts_without = contexts(&[world(&empty.0, &[])], &INSTALLED, &[Probe::Beta], |_| {
        None
    })
    .expect("beta's shipped files wait for an owner");
    assert!(contexts_without.is_empty());

    let with_alpha = contexts(&[world(&empty.0, &[])], &INSTALLED, &[Probe::Alpha], |_| {
        None
    })
    .expect("alpha alone");
    assert_eq!(
        seen(&with_alpha),
        [(
            "alpha".to_owned(),
            DataSource::SystemPack,
            "one/self.yaml".to_owned()
        )]
    );
}

/// The loader's format directories are the ones no pack may own, and in this build no two installed
/// packs own one directory — a structural guard over the closed catalog, as the section names' is.
#[test]
fn the_format_directories_agree_and_no_two_packs_own_one_directory() {
    let mut format: Vec<&str> = WORLD_DIRECTORIES.to_vec();
    let mut reserved: Vec<&str> = DataDirectory::FORMAT.to_vec();
    format.sort_unstable();
    reserved.sort_unstable();
    assert_eq!(format, reserved);

    let mut owned: Vec<&str> = AVAILABLE
        .into_iter()
        .flat_map(|capability| DataOwnerOps::directories(capability).iter())
        .map(|directory| directory.as_str())
        .collect();
    let count = owned.len();
    owned.sort_unstable();
    owned.dedup();
    assert_eq!(owned.len(), count, "two packs own one directory: {owned:?}");
}

/// A decoded configuration that only yields files (S31-b2 §5.1): what a pack's configuration hands the
/// loader through [`AuthoredConfiguration::yields`].
#[derive(Debug)]
struct Yielding {
    owner: SystemId,
    files: Vec<YieldedFile>,
}

impl AuthoredConfiguration for Yielding {
    fn owner(&self) -> SystemId {
        self.owner.clone()
    }

    fn references(&self) -> Vec<mineworld_authoring::Reference<'_>> {
        Vec::new()
    }

    fn requires(&self) -> Vec<SystemId> {
        Vec::new()
    }

    fn facts(&self) -> &'static [mineworld_contracts::EventTypeId] {
        &[]
    }

    fn attachments(&self) -> Vec<&mineworld_authoring::Attachment> {
        Vec::new()
    }

    fn check(
        &self,
        _: &mineworld_authoring::ConfigurationContext<'_>,
    ) -> Result<(), mineworld_authoring::ConfigurationRefusal> {
        Ok(())
    }

    fn yields(&self) -> Vec<YieldedFile> {
        self.files.clone()
    }

    fn seed(
        &self,
        _: &mineworld_authoring::Seeding<'_, '_>,
        _: &mineworld_authoring::ConfigurationContext<'_>,
    ) -> Result<Vec<mineworld_kernel::Emission>, mineworld_contracts::Rejection> {
        Ok(Vec::new())
    }
}

/// Beta's configuration, yielding `files`.
fn beta_yields(files: &[(DataDirectory, &str, &str)]) -> Yielding {
    Yielding {
        owner: SystemId::from_static("beta"),
        files: files
            .iter()
            .map(|(directory, name, text)| YieldedFile {
                directory: *directory,
                name: (*name).to_owned(),
                text: (*text).to_owned(),
            })
            .collect(),
    }
}

/// The full paths Alpha's context saw, in order.
fn seen_paths(
    contexts: &BTreeMap<Probe, Arc<dyn OwnerContext>>,
) -> Vec<(String, DataSource, String)> {
    contexts[&Probe::Alpha]
        .as_any()
        .downcast_ref::<Vec<Seen>>()
        .expect("alpha's context")
        .iter()
        .map(|seen| (seen.pack.clone(), seen.source, seen.path.clone()))
        .collect()
}

/// S31-b2 C1: a configuration's yielded files are collected like shipped ones — after the yielding
/// pack's shipped files, by directory then name, with the configuring pack as their origin and the
/// shown path `configure/<pack>.yaml → <directory>/<name>` — and a world whose packs yield nothing
/// collects exactly what it did before.
#[test]
fn yielded_files_are_collected_with_the_configuring_pack() {
    let empty = Scratch::new("yield-order");
    let roots = [world(&empty.0, &[])];
    let configuration = beta_yields(&[
        (TWO, "t.yaml", "from: beta-configuration\n"),
        (ONE, "y.yaml", "from: beta-configuration\n"),
    ]);
    let collected = contexts(&roots, &INSTALLED, &[Probe::Beta, Probe::Alpha], |owner| {
        (owner == Probe::Beta).then_some(&configuration as &dyn AuthoredConfiguration)
    })
    .expect("collects");
    use DataSource::SystemPack;
    let expected: Vec<(String, DataSource, String)> = [
        ("beta", "one/a.yaml"),
        ("beta", "one/z.yaml"),
        ("beta", "two/b.yaml"),
        ("beta", "configure/beta.yaml → one/y.yaml"),
        ("beta", "configure/beta.yaml → two/t.yaml"),
        ("alpha", "one/self.yaml"),
    ]
    .into_iter()
    .map(|(pack, path)| (pack.to_owned(), SystemPack, path.to_owned()))
    .collect();
    assert_eq!(seen_paths(&collected), expected);

    let unchanged =
        contexts(&roots, &INSTALLED, &[Probe::Beta, Probe::Alpha], |_| None).expect("collects");
    assert_eq!(
        seen_paths(&unchanged),
        expected[..3]
            .iter()
            .chain(&expected[5..])
            .cloned()
            .collect::<Vec<_>>()
    );
}

/// S31-b2 C1: a yielded file for an owner the world does not enable is skipped, as a shipped one is.
#[test]
fn a_yielded_file_for_an_owner_not_enabled_is_skipped() {
    let empty = Scratch::new("yield-skip");
    let configuration = beta_yields(&[(ONE, "y.yaml", "from: beta-configuration\n")]);
    let collected = contexts(
        &[world(&empty.0, &[])],
        &INSTALLED,
        &[Probe::Beta],
        |owner| (owner == Probe::Beta).then_some(&configuration as &dyn AuthoredConfiguration),
    )
    .expect("alpha is not enabled: nothing is read");
    assert!(collected.is_empty());
}

/// S31-b2 C1: every refusal about a yielded file names the configuring pack and the shown path — a
/// directory nobody owns, a malformed file, a bad name, and the owner's own refusal.
#[test]
fn every_yielded_refusal_names_the_configuring_pack() {
    let empty = Scratch::new("yield-refusals");
    let roots = [world(&empty.0, &[])];
    let refuse = |files: &[(DataDirectory, &str, &str)]| {
        let configuration = beta_yields(files);
        contexts(&roots, &INSTALLED, &[Probe::Beta, Probe::Alpha], |owner| {
            (owner == Probe::Beta).then_some(&configuration as &dyn AuthoredConfiguration)
        })
        .expect_err("refused")
    };

    let nowhere = DataDirectory::from_static("nowhere");
    let refusal = refuse(&[(nowhere, "x.yaml", "a: b\n")]);
    let PackError::UnknownDirectory { path, known } = &refusal else {
        panic!("got: {refusal}");
    };
    assert_eq!(*path, PathBuf::from("configure/beta.yaml → nowhere/x.yaml"));
    assert_eq!(known, "one, two");

    let refusal = refuse(&[(ONE, "bad.yaml", "a: b\nc: [unclosed\n")]);
    let PackError::Malformed { path, kind, detail } = &refusal else {
        panic!("got: {refusal}");
    };
    assert_eq!(*path, PathBuf::from("configure/beta.yaml → one/bad.yaml"));
    assert_eq!(*kind, "data");
    assert!(detail.contains("line"), "a position: {detail}");

    let refusal = refuse(&[(ONE, "../escape.yaml", "a: b\n")]);
    assert!(
        matches!(&refusal, PackError::Malformed { path, .. } if path.to_string_lossy().contains("configure/beta.yaml")),
        "got: {refusal}"
    );

    let refusal = refuse(&[(ONE, "r.yaml", "refuse: the owner says no\n")]);
    let PackError::DataRefused {
        system,
        file,
        reason,
    } = &refusal
    else {
        panic!("got: {refusal}");
    };
    assert_eq!(*system, AlphaData::ID);
    assert_eq!(
        file.as_deref(),
        Some(Path::new("configure/beta.yaml → one/r.yaml"))
    );
    assert_eq!(reason, "the owner says no");
}
