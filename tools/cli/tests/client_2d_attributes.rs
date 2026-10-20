//! S30-f §5.11, the 2D surface (`pr-s30f-attribute-consumers.md` AC-f-1 … AC-f-3): in the fixture world
//! `attributes-consumers`, Bob's 2D client opens Alice's menu and shows her glance under the title and her
//! inspect list below it; Alice's own panels show her sheet — each equal to the hand-written oracle, with
//! nothing of Alice's private attributes on Bob's screen.
//!
//! Every test here starts Godot, so it is `#[ignore]`d: `cargo test` reports it as ignored, never as passed.
//!
//! ```text
//! cargo test -p mineworld-cli --test client_2d_attributes -- --ignored --test-threads=1
//! ```

mod godot2d;
mod headless;
mod support;

use godot2d::worlds::{menus, panel, play};
use godot2d::{Drive, World, passed, tagged};
use mineworld_contracts::{Component, EntityType};
use mineworld_naming::DisplayName;
use serde_json::{Value, json};
use support::Client;

const WORLD: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/attributes-consumers"
);
const ORACLE: &str = include_str!("fixtures/attributes-consumers.expected.json");
const ENTITIES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../entities");

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .expect("a list")
        .iter()
        .map(|item| item.as_str().expect("a string").to_owned())
        .collect()
}

/// Alice's identity as a client writes it, read from Carol's observation through the real server.
async fn alices_id(world: &World) -> String {
    let mut carol = Client::connect(world.address).await;
    carol.join("carol").await;
    let seen = carol.observation().await;
    seen.entities()
        .iter()
        .filter(|entity| matches!(entity.entity_type(), EntityType::Person))
        .find(|entity| {
            entity.components().iter().any(|record| {
                *record.component_type() == DisplayName::COMPONENT_TYPE
                    && record.payload().get("name").and_then(Value::as_str) == Some("Alice Moreau")
            })
        })
        .map(|entity| entity.id().raw().to_string())
        .expect("Carol perceives Alice")
}

#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_attributes -- --ignored"]
fn bob_sees_alices_glance_and_inspect_list_and_alice_her_own_sheet() {
    let runtime = tokio::runtime::Runtime::new().expect("a runtime");
    runtime.block_on(async {
        let oracle: Value = serde_json::from_str(ORACLE).expect("the oracle is JSON");
        let world = World::start(&["server", WORLD, "--packs", ENTITIES], None).await;
        let alice = alices_id(&world).await;

        let steps = json!([{ "open": alice }, { "report": true }]);
        let (status, lines) = play(world.address, "bob", "s30f-2d-bob", &steps, &[]).await;
        assert!(status.success() && passed(&lines), "Bob's drive passed");
        let menu = menus(&lines, &alice).pop().expect("Alice's menu was drawn");
        assert_eq!(
            menu["glance"], oracle["alice_seen_by_bob"]["glance"],
            "the menu header"
        );
        assert_eq!(
            strings(&menu["inspect"]),
            strings(&oracle["alice_seen_by_bob"]["inspect"]),
            "the subject panel"
        );
        let screen = lines.join("\n");
        for needle in strings(&oracle["absent_from_bob"]["texts"]) {
            assert!(
                !screen.contains(&needle),
                "Bob's 2D client shows '{needle}'"
            );
        }

        let drive = Drive::start(world.address, "alice", &["--drive=panels"]);
        let (status, lines) = drive.finish().await;
        assert!(status.success(), "Alice's panels drive ran");
        let last = tagged(&lines, "PANELS ")
            .into_iter()
            .rfind(|record| record["final"] == true)
            .expect("a final PANELS line");
        let mut shown = strings(&panel(&last, "traits").expect("the traits panel")["rows"]);
        let work = strings(&panel(&last, "employment").expect("the work panel")["rows"]);
        let mut expected = strings(&oracle["alice_own"]["sheet"]);
        // The work panel's own row first, then the sheet of the attributes employment owns.
        shown.extend(work.into_iter().filter(|row| expected.contains(row)));
        shown.sort();
        expected.sort();
        assert_eq!(
            shown, expected,
            "Alice's own sheet, across her traits and work panels"
        );
    });
}
