//! A town's far field fills its slots with the places the town declares, at the authoring gate (S29-c §8,
//! AC-c-2 … AC-c-4; `ARC-107`). Every world here is a reference world or a generated copy in scratch, and the
//! 3D packs are generated: a `pack.yaml`, a style manifest and one far-field file, so the slots are the only
//! thing under test.

use std::path::{Path, PathBuf};
use std::process::Command;

use mineworld_test_support::Scratch;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const THREE_D: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../presentation/mineworld-default/3D"
);

fn mineworld(arguments: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_mineworld"))
        .args(arguments)
        .output()
        .expect("the binary runs");
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("writable");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copied");
        }
    }
}

fn replace_in(file: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(file).expect("reads");
    assert_eq!(
        text.matches(from).count(),
        1,
        "{}: `{from}` once",
        file.display()
    );
    std::fs::write(file, text.replace(from, to)).expect("writes");
}

/// A 3D Presentation Pack in `root/<id>/` with only its identity, its style manifest and `far_field` for
/// `world`, read from the shipped one and rewritten by `edit`.
fn generated_pack(root: &Path, id: &str, world: &str, field: &str) -> PathBuf {
    let dir = root.join(id);
    std::fs::create_dir_all(dir.join("far_field")).expect("writable");
    let identity = std::fs::read_to_string(Path::new(THREE_D).join("pack.yaml")).expect("reads");
    std::fs::write(
        dir.join("pack.yaml"),
        identity.replace("id: mineworld-default-3d", &format!("id: {id}")),
    )
    .expect("writes");
    std::fs::copy(
        Path::new(THREE_D).join("manifest.yaml"),
        dir.join("manifest.yaml"),
    )
    .expect("copied");
    std::fs::write(dir.join("far_field").join(format!("{world}.yaml")), field).expect("writes");
    dir
}

fn shipped_field(world: &str) -> String {
    std::fs::read_to_string(
        Path::new(THREE_D)
            .join("far_field")
            .join(format!("{world}.yaml")),
    )
    .expect("the shipped far field reads")
}

fn validate(world: &Path, packs: &Path) -> (bool, String) {
    let world = world.to_str().expect("printable");
    let packs = packs.to_str().expect("printable");
    let (ok, said) = mineworld(&["validate", world, "--packs", packs]);
    let (ok_packs, said_packs) = mineworld(&["packs", "validate", world, "--packs", packs]);
    assert_eq!(ok, ok_packs, "both commands agree: {said} / {said_packs}");
    (ok, format!("{said}{said_packs}"))
}

fn scratch(name: &str) -> Scratch {
    mineworld_test_support::scratch!(empty name)
}

#[test]
fn the_reference_towns_pass_with_their_filled_slots() {
    let entities = format!("{ROOT}/entities");
    let presentation = format!("{ROOT}/presentation/mineworld-default");
    for world in ["social-cafe", "lakeside", "market-town", "bodies-yard"] {
        let path = format!("{ROOT}/worlds/{world}");
        for command in [
            vec!["validate", path.as_str()],
            vec!["packs", "validate", path.as_str()],
        ] {
            let mut arguments = command.clone();
            arguments.extend(["--packs", &entities, "--packs", &presentation]);
            let (ok, said) = mineworld(&arguments);
            assert!(ok, "{world}: {said}");
        }
    }
}

#[test]
fn a_generated_town_is_held_to_its_own_keys() {
    // A copy of Social Café whose landmark is the park: the far field must name `park`, not `station`.
    let root = scratch("far-field-slots-park");
    let world = root.join("social-cafe");
    copy_dir(&Path::new(ROOT).join("worlds/social-cafe"), &world);
    replace_in(
        &world.join("world.yaml"),
        "  landmark: station\n",
        "  landmark: park\n",
    );
    replace_in(
        &world.join("places/park.yaml"),
        "tags:\n",
        "tags:\n  - landmark\n",
    );
    let packs = root.join("packs");
    let shipped = shipped_field("social-cafe");
    let landmark = shipped
        .split("\"landmark\": {")
        .nth(1)
        .expect("a landmark slot")
        .to_owned();
    let renamed = landmark.replacen("\"place\": \"station\"", "\"place\": \"park\"", 1);
    generated_pack(
        &packs,
        "gen-3d",
        "social-cafe",
        &shipped.replace(&landmark, &renamed),
    );
    let (ok, said) = validate(&world, &packs);
    assert!(ok, "the generated town passes with its own keys: {said}");

    // The same pack against the unchanged world refuses: the far field names `park`, the world `station`.
    let plain = root.join("plain");
    let original = plain.join("social-cafe");
    copy_dir(&Path::new(ROOT).join("worlds/social-cafe"), &original);
    let (ok, said) = validate(&original, &packs);
    assert!(!ok, "{said}");
    assert!(
        said.contains("FAR_FIELD_SLOT_TOWN") && said.contains("slot landmark"),
        "{said}"
    );
}

#[test]
fn a_town_with_one_or_no_filled_slot_is_refused() {
    let root = scratch("far-field-slots-town");
    let shipped = shipped_field("social-cafe");
    let station = shipped
        .split("\"landmark\": {")
        .next()
        .expect("the station half")
        .to_owned();
    let reserved_station = station.replacen(
        "\"state\": \"filled\", \"place\": \"station\", \"bearing_deg\": 270, \"distance_m\": 68",
        "\"state\": \"reserved\", \"bearing_deg\": 270, \"distance_m\": 790",
        1,
    );
    // X-4: the station set back to reserved, the landmark filled → the file-level rule.
    let one = root.join("one");
    generated_pack(
        &one,
        "gen-3d",
        "social-cafe",
        &shipped.replace(&station, &reserved_station),
    );
    let world = Path::new(ROOT).join("worlds/social-cafe");
    let (ok, said) = validate(&world, &one);
    assert!(
        !ok && said.contains("FAR_FIELD_SLOT_TOWN") && said.contains("only landmark is filled"),
        "{said}"
    );

    // Both reserved → the world-level rule names the station slot and the world.
    let landmark = shipped
        .split("\"landmark\": {")
        .nth(1)
        .expect("the landmark half")
        .to_owned();
    let reserved_landmark = landmark.replacen(
        "\"state\": \"filled\", \"place\": \"station\", \"bearing_deg\": 270, \"distance_m\": 68",
        "\"state\": \"reserved\", \"bearing_deg\": 270, \"distance_m\": 790",
        1,
    );
    let none = root.join("none");
    let both = shipped
        .replace(&station, &reserved_station)
        .replace(&landmark, &reserved_landmark);
    generated_pack(&none, "gen-3d", "social-cafe", &both);
    let (ok, said) = validate(&world, &none);
    assert!(!ok, "{said}");
    assert!(
        said.contains("FAR_FIELD_SLOT_TOWN") && said.contains("world social-cafe, slot station"),
        "{said}"
    );
}

#[test]
fn a_filled_slot_in_a_world_that_is_not_a_town_is_refused() {
    let root = scratch("far-field-slots-yard");
    let field = shipped_field("social-cafe").replace(
        "\"scope\": {\"world\": \"social-cafe\"}",
        "\"scope\": {\"world\": \"bodies-yard\"}",
    );
    generated_pack(&root, "gen-3d", "bodies-yard", &field);
    let world = Path::new(ROOT).join("worlds/bodies-yard");
    let (ok, said) = validate(&world, &root);
    assert!(!ok, "{said}");
    assert!(
        said.contains("FAR_FIELD_SLOT:") && said.contains("slot station"),
        "{said}"
    );
}
