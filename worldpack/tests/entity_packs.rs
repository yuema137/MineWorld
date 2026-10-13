//! Entity Packs read by the World Pack loader (`DECISIONS.md` `ARC-71`; step-16 §17.5 ED-2, ED-3, ED-7,
//! ED-8, ED-10, ED-13): a required pack's item kinds join the world's as if the world had declared them
//! — one namespace, one allocation order, the pack named as their provenance — and every way that can go
//! wrong is refused by name, on every platform and with either line ending.
//!
//! The ED-1 fixture is Market Town with `bread` and `coffee` moved out of its `items:` and `items/` into
//! an Entity Pack `goods` in a scratch pack root, required as `goods: "^0.1"`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mineworld_contracts::{EntityId, EntityKey, WorldTime};
use mineworld_worldpack::{LoadedWorld, PackError, PackRoots, WorldPack, validate_entity_pack};

const REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");

const GOODS: &str = "id: goods\ntype: entity-pack\nversion: 0.1.0\nmineworld: \"^0.1\"\nlicense: MIT\n\
                     authors: [Someone]\n";

fn key(text: &str) -> EntityKey {
    EntityKey::new(text).expect("a key")
}

fn market_town() -> PathBuf {
    Path::new(REPOSITORY).join("worlds").join("market-town")
}

/// `text` with every line ending made `\n`, then `\r\n` when `crlf`: a checkout's own endings (Git's
/// `autocrlf` on Windows) never decide which case a test is.
fn endings(text: &str, crlf: bool) -> String {
    let lf = text.replace("\r\n", "\n");
    if crlf { lf.replace('\n', "\r\n") } else { lf }
}

/// A scratch directory holding a world and a pack root `root/`, every YAML file written with one line
/// ending. Removed when the test ends (DEP-29).
struct Scratch {
    base: mineworld_test_support::Scratch,
    crlf: bool,
}

impl Scratch {
    fn new(name: &str, crlf: bool) -> Self {
        let base = mineworld_test_support::scratch!(format!("entity-packs-{name}"));
        std::fs::create_dir_all(base.join("root")).expect("root");
        Self { base, crlf }
    }

    fn path(&self, relative: &str) -> PathBuf {
        relative
            .split('/')
            .fold(self.base.to_path_buf(), |path, part| path.join(part))
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.path(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("parent directory");
        std::fs::write(path, endings(text, self.crlf)).expect("file writes");
    }

    /// A pack `id` in `root/` holding `items` (`(key, text)`), stating `pack` as its `pack.yaml`.
    fn pack(&self, id: &str, pack: &str, items: &[(&str, &str)]) {
        self.write(&format!("root/{id}/pack.yaml"), pack);
        for (key, text) in items {
            self.write(&format!("root/{id}/items/{key}.yaml"), text);
        }
    }

    fn goods(&self, items: &[(&str, &str)]) {
        self.pack("goods", GOODS, items);
    }

    fn roots(&self) -> PackRoots {
        PackRoots::new(vec![self.path("root")], None).expect("the root exists")
    }

    fn read(&self, world: &str) -> Result<WorldPack, Box<PackError>> {
        WorldPack::read_with(self.path(world), &self.roots()).map_err(Box::new)
    }

    /// Market Town, copied into `market-town/` with every YAML file in this scratch's line ending, minus
    /// the item kinds `moved` (from `items:` and `items/`), plus `top` appended to `world.yaml`. The
    /// moved kinds' files are returned, for a pack to hold.
    fn market_town(&self, moved: &[&str], top: &str) -> Vec<(String, String)> {
        copy_tree(&market_town(), &self.path("market-town"), self.crlf);
        let manifest_path = self.path("market-town/world.yaml");
        let mut manifest = endings(
            &std::fs::read_to_string(&manifest_path).expect("world.yaml"),
            false,
        );
        let mut files = Vec::new();
        for kind in moved {
            let line = format!("  - {kind}\n");
            assert!(
                manifest.contains(&line),
                "{kind} is one of Market Town's kinds"
            );
            manifest = manifest.replacen(&line, "", 1);
            let file = self.path(&format!("market-town/items/{kind}.yaml"));
            files.push((
                (*kind).to_owned(),
                endings(&std::fs::read_to_string(&file).expect("its file"), false),
            ));
            std::fs::remove_file(file).expect("moved out");
        }
        self.write("market-town/world.yaml", &format!("{manifest}{top}"));
        files
    }

    /// ED-1's fixture: bread and coffee moved into `goods`, which the town requires.
    fn moved_town(&self) {
        let files = self.market_town(&["bread", "coffee"], "requires:\n  goods: \"^0.1\"\n");
        let files: Vec<(&str, &str)> = files
            .iter()
            .map(|(key, text)| (key.as_str(), text.as_str()))
            .collect();
        self.goods(&files);
    }

    /// A one-place world `the-world` enabling `systems`, with `top` appended.
    fn small_world(&self, systems: &str, top: &str) {
        self.write(
            "the-world/world.yaml",
            &format!(
                "world:\n  id: the-world\n  name: Small\nsystems: [{systems}]\nplaces: [cafe]\n\
                 population: [alice]\n{top}"
            ),
        );
        self.write("the-world/places/cafe.yaml", "tags: [cafe]\n");
        self.write(
            "the-world/people/alice.yaml",
            "tags: [barista]\nlocation:\n  place: cafe\n",
        );
    }
}

/// Copies `from` into `to`, rewriting every `.yaml` file's line endings; other files are copied as bytes.
fn copy_tree(from: &Path, to: &Path, crlf: bool) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("a readable directory") {
        let path = entry.expect("an entry").path();
        let target = to.join(path.file_name().expect("a name"));
        if path.is_dir() {
            copy_tree(&path, &target, crlf);
        } else if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            let text = std::fs::read_to_string(&path).expect("UTF-8 YAML");
            std::fs::write(&target, endings(&text, crlf)).expect("written");
        } else {
            std::fs::copy(&path, &target).expect("copied");
        }
    }
}

fn load(pack: &WorldPack) -> LoadedWorld {
    pack.load(WorldTime::EPOCH).expect("loads")
}

/// Every genesis fact as `(id, type, payload)`: what a replay compares.
fn genesis(world: &LoadedWorld) -> Vec<(String, String, String)> {
    world
        .genesis()
        .iter()
        .map(|fact| {
            (
                format!("{:?}", fact.id()),
                fact.event_type().to_string(),
                String::from_utf8_lossy(fact.payload().payload()).into_owned(),
            )
        })
        .collect()
}

fn ids(world: &LoadedWorld) -> BTreeMap<EntityKey, EntityId> {
    world.ids().clone()
}

/// ED-2 with ED-1's equality at the loader: moving two kinds into a required Entity Pack changes no
/// identity and no genesis fact, and names the pack as their provenance (`ARC-71` points 6, 7).
#[test]
fn kinds_moved_into_a_pack_keep_every_identity_and_name_the_pack() {
    let scratch = Scratch::new("moved", false);
    scratch.moved_town();
    let moved = scratch.read("market-town").expect("reads with goods");
    let original = WorldPack::read(market_town()).expect("Market Town reads");
    assert_eq!(
        moved.items().keys().collect::<Vec<_>>(),
        original.items().keys().collect::<Vec<_>>(),
        "the composed kinds are the town's twenty"
    );

    let (moved, original) = (load(&moved), load(&original));
    assert_eq!(ids(&moved), ids(&original), "every key keeps its identity");
    assert!(
        !genesis(&original).is_empty(),
        "the comparison compares something"
    );
    // Located before counted: a genesis `stocked` fact names the pack's kind by its identity.
    let bread_id = moved.id(&key("bread")).expect("bread resolves");
    let named = format!("\"item\":{{\"entity\":\"{}\"", bread_id.raw());
    assert!(
        genesis(&moved)
            .iter()
            .any(|(_, event_type, payload)| event_type == "stocked" && payload.contains(&named)),
        "a stocked fact names bread ({named})"
    );
    assert_eq!(
        genesis(&moved),
        genesis(&original),
        "every genesis fact unchanged"
    );

    let read = moved.world().read();
    let provenance = |name: &str| {
        read.entity(moved.id(&key(name)).expect("resolves"))
            .expect("exists")
            .metadata()
            .expect("authored")
            .clone()
    };
    let bread = provenance("bread");
    assert_eq!(bread.source_pack, "goods");
    assert_eq!(
        bread.source_path, "items/bread.yaml",
        "written with '/' on every OS"
    );
    assert_eq!(provenance("apple").source_pack, "market-town");
    assert_eq!(provenance("apple").source_path, "items/apple.yaml");
}

/// ED-13 (PD-q1 … PD-q3): the same pack and world written with CRLF read exactly as with LF — the same
/// kinds, composition, identities and genesis — and a malformed CRLF item file is refused naming the file
/// and the line.
#[test]
fn crlf_is_content_not_an_error() {
    let lf = Scratch::new("lf", false);
    lf.moved_town();
    let crlf = Scratch::new("crlf", true);
    crlf.moved_town();
    let crlf_bytes = std::fs::read(crlf.path("root/goods/items/bread.yaml")).expect("bread");
    assert!(
        crlf_bytes.windows(2).any(|pair| pair == b"\r\n"),
        "the case is CRLF"
    );

    let (a, b) = (
        lf.read("market-town").expect("LF reads"),
        crlf.read("market-town").expect("CRLF reads"),
    );
    assert_eq!(
        a.items().keys().collect::<Vec<_>>(),
        b.items().keys().collect::<Vec<_>>()
    );
    let required = |pack: &WorldPack| {
        pack.composition()
            .required
            .iter()
            .map(|r| (r.identity.clone(), r.range.to_string()))
            .collect::<Vec<_>>()
    };
    assert_eq!(required(&a), required(&b), "the same composition");
    let (a, b) = (load(&a), load(&b));
    assert_eq!(ids(&a), ids(&b));
    assert_eq!(genesis(&a), genesis(&b));

    crlf.write(
        "root/goods/items/bread.yaml",
        "tags: [food\nitem: { category: food }\n",
    );
    let refusal = crlf.read("market-town").expect_err("malformed").to_string();
    let file = crlf
        .path("root/goods/items/bread.yaml")
        .display()
        .to_string();
    assert!(
        refusal.contains(&file) && refusal.contains("line"),
        "names the file and the line: {refusal}"
    );
}

/// ED-3 (`ARC-71` point 5): one key namespace, refused by name, never "last one wins".
#[test]
fn a_key_from_two_sources_is_refused_naming_both() {
    let scratch = Scratch::new("collision-world", false);
    scratch.market_town(&[], "requires:\n  goods: \"^0.1\"\n");
    scratch.goods(&[("bread", "tags: [food]\n")]);
    let refusal = scratch
        .read("market-town")
        .expect_err("bread twice")
        .to_string();
    let file = scratch
        .path("root/goods/items/bread.yaml")
        .display()
        .to_string();
    for needle in [
        "'bread'",
        "world.yaml's items",
        "the Entity Pack goods",
        &file,
    ] {
        assert!(refusal.contains(needle), "{needle:?} not in {refusal}");
    }

    let scratch = Scratch::new("collision-packs", false);
    scratch.market_town(
        &["tea"],
        "requires:\n  goods: \"^0.1\"\n  more-goods: \"^0.1\"\n",
    );
    scratch.goods(&[("tea", "tags: [drink]\n")]);
    scratch.pack(
        "more-goods",
        &GOODS.replace("id: goods", "id: more-goods"),
        &[("tea", "tags: [drink]\n")],
    );
    let refusal = scratch
        .read("market-town")
        .expect_err("tea twice")
        .to_string();
    for needle in [
        "'tea'",
        "the Entity Pack goods",
        "the Entity Pack more-goods",
    ] {
        assert!(refusal.contains(needle), "{needle:?} not in {refusal}");
    }
}

/// ED-7: a pack's `item:` section belongs to `item`, which the requiring world must enable — refused
/// naming the pack's file and the section, exactly as a world's own file would be (rule 6).
#[test]
fn a_pack_section_needs_its_owner_enabled_by_the_world() {
    let scratch = Scratch::new("owner", false);
    scratch.small_world("presence", "requires:\n  goods: \"^0.1\"\n");
    scratch.goods(&[("bread", "tags: [food]\nitem: { category: food }\n")]);
    let refusal = scratch
        .read("the-world")
        .expect_err("item not enabled")
        .to_string();
    let file = scratch
        .path("root/goods/items/bread.yaml")
        .display()
        .to_string();
    for needle in [file.as_str(), "item", "does not enable it"] {
        assert!(refusal.contains(needle), "{needle:?} not in {refusal}");
    }
}

/// ED-8 (`ARC-71` point 8; FQ-d3): an Entity Pack is self-contained — a `body:` lying `at:` a world's
/// place is refused naming the pack, the file and the key, both when a world requires it and when
/// `packs validate` reads it against the whole installed set.
#[test]
fn a_pack_section_naming_a_key_outside_the_pack_is_refused() {
    let scratch = Scratch::new("reaches-out", false);
    scratch.small_world("presence, bodies", "requires:\n  goods: \"^0.1\"\n");
    scratch.goods(&[(
        "lamp",
        "tags: [light]\nbody:\n  shape: { ball: 110 }\n  at: { place: cafe, x: 0, y: 0 }\n",
    )]);
    let file = scratch
        .path("root/goods/items/lamp.yaml")
        .display()
        .to_string();
    let world = scratch
        .read("the-world")
        .expect_err("reaches out")
        .to_string();
    let validated = validate_entity_pack(&scratch.path("root/goods"))
        .expect_err("reaches out")
        .to_string();
    for refusal in [world, validated] {
        for needle in [file.as_str(), "goods", "'cafe'"] {
            assert!(refusal.contains(needle), "{needle:?} not in {refusal}");
        }
    }
}

/// ED-10: the pack's own refusals, each by name — and the frozen model's names stay refused.
#[test]
fn every_bad_entity_pack_is_refused_by_name() {
    type Case = (
        &'static str,
        &'static [(&'static str, &'static str)],
        &'static [&'static str],
    );
    let cases: [Case; 4] = [
        (
            "an item file whose name is not a key",
            &[("Bread", "tags: [food]\n")],
            &["Bread.yaml", "goods", "not a key"],
        ),
        (
            "a malformed item file",
            &[("bread", "tags: [food]\ncolour: brown\n")],
            &["bread.yaml", "colour", "line"],
        ),
        (
            "a people/ directory",
            &[("bread", "tags: [food]\n")],
            &["people", "item kinds only in MVP-0"],
        ),
        ("no item file", &[], &["declares nothing"]),
    ];
    for (case, items, needles) in cases {
        let name: String = case
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        let scratch = Scratch::new(&format!("bad-{name}"), false);
        scratch.small_world("presence", "requires:\n  goods: \"^0.1\"\n");
        scratch.goods(items);
        if case == "a people/ directory" {
            scratch.write("root/goods/people/alice.yaml", "tags: [x]\n");
        }
        if items.is_empty() {
            scratch.write("root/goods/items/README.md", "not content\n");
        }
        let refusal = scratch.read("the-world").expect_err(case).to_string();
        for needle in needles {
            assert!(
                refusal.contains(needle),
                "{case}: {needle:?} not in {refusal}"
            );
        }
    }

    let scratch = Scratch::new("bad-frozen-names", false);
    scratch.small_world("presence", "entity_packs: [goods]\n");
    let refusal = scratch
        .read("the-world")
        .expect_err("entity_packs")
        .to_string();
    assert!(refusal.contains("entity_packs"), "{refusal}");
    scratch.small_world("presence", "requires:\n  goods: \"^0.1\"\n");
    scratch.goods(&[("bread", "tags: [food]\n")]);
    scratch.write(
        "root/goods/pack.yaml",
        &format!("{GOODS}dependencies: {{ other: \"^0.1\" }}\n"),
    );
    let refusal = scratch
        .read("the-world")
        .expect_err("dependencies")
        .to_string();
    assert!(refusal.contains("dependencies"), "{refusal}");
}

/// `packs validate`'s reader (`ARC-71` point 9): a good pack's kinds, in key order, read against the
/// whole installed set.
#[test]
fn packs_validate_lists_the_kinds_of_a_good_pack() {
    let scratch = Scratch::new("validate", false);
    scratch.moved_town();
    let kinds = validate_entity_pack(&scratch.path("root/goods")).expect("valid");
    assert_eq!(kinds, [key("bread"), key("coffee")]);
}
