//! S34-e: real 2D click -> walk -> sit -> floor click -> requested stand, in both café worlds.
//! The server's persisted facts and the client's outgoing requests are independent oracles.

mod godot2d;
mod godot3d;
mod headless;
mod support;

use godot2d::worlds::{facts, start};
use godot2d::{World, passed, tagged};
use mineworld_contracts::ActionRequest;
use mineworld_persistence::{JournalEntry, WorldInput, format};
use mineworld_server::{RequestField, WirePayload, differing_fields};
use serde_json::json;
use support::SaveDir;

#[tokio::test]
#[ignore = "requires Godot; run at the S34-e milestone"]
async fn click_walk_sit_and_requested_stand_in_both_worlds() {
    for key in ["social-cafe", "market-town"] {
        let pack = format!("{}/../../worlds/{key}", env!("CARGO_MANIFEST_DIR"));
        let save = SaveDir::new(&format!("s34e-{key}"));
        let world = World::start(&["server", &pack, "--save", save.path()], None).await;
        let steps = json!([
            {"record_seating": true},
            {"click_sit": "nearest-far"},
            {"await": {"component": "posture-state"}},
            {"report_posture": true},
            {"record_seating": true},
            {"open": "self"},
            {"key": "esc"},
            {"floor_near_seat": true},
            {"await": {"component": "posture-state", "present": false}},
            {"report_posture": true},
            {"record_seating": true},
            {"click_sit": "last"},
            {"await": {"component": "posture-state"}},
            {"report_posture": true},
            {"record_seating": true}
        ]);
        let world_key = format!("--world-key={key}");
        let (drive, _steps) = start(
            world.address,
            "visitor",
            &format!("s34e-steps-{key}"),
            &steps,
            &["--presentation=none", "--settings=none", &world_key],
        );
        let (status, lines) = drive.finish().await;
        assert!(status.success() && passed(&lines), "{key}: {lines:#?}");
        let requests = tagged(&lines, "REQUEST ");
        let actions: Vec<_> = requests
            .iter()
            .filter_map(|r| r["request"]["action_type"].as_str())
            .collect();
        let sit = actions.iter().position(|a| *a == "sit").expect("sit sent");
        let stand = actions
            .iter()
            .position(|a| *a == "stand")
            .expect("stand sent");
        assert!(
            actions[..sit].contains(&"walk-to"),
            "walk precedes sit: {actions:?}"
        );
        assert!(
            sit < stand && actions[stand + 1..].contains(&"walk-to"),
            "stand precedes leaving: {actions:?}"
        );
        assert_eq!(
            actions.iter().filter(|a| **a == "sit").count(),
            2,
            "one try per click"
        );
        let second = actions.iter().rposition(|a| *a == "sit").unwrap();
        let second_step = godot2d::worlds::step_line(&lines, 11);
        let after_click = tagged(&lines[second_step..], "REQUEST ");
        assert!(
            second > stand
                && after_click
                    .iter()
                    .all(|r| r["request"]["action_type"] != "walk-to"),
            "within-reach click must not walk: {after_click:?}"
        );
        let sent_offers = tagged(&lines, "SIT_SENT ");
        assert_eq!(
            sent_offers.len(),
            2,
            "each sent sit has current-offer evidence"
        );
        for sent in sent_offers {
            let request = requests
                .iter()
                .find(|r| r["token"] == sent["token"])
                .unwrap();
            let offer = sent["offers"]
                .as_array()
                .unwrap()
                .iter()
                .find(|o| o["payload"]["point"] == sent["point"])
                .expect("offered point");
            let actual: ActionRequest<WirePayload> =
                serde_json::from_value(godot3d::whole(&request["request"])).unwrap();
            let expected: ActionRequest<WirePayload> = serde_json::from_value(godot3d::whole(&json!({
                "actor": actual.actor(), "action_type": offer["action_type"], "target": offer["target"],
                "payload": {"action_type": offer["action_type"], "payload": offer["payload"]}, "actor_location": null
            }))).unwrap();
            let differences = differing_fields(&actual, &expected);
            assert!(
                differences
                    .iter()
                    .all(|f| *f == RequestField::ActorLocation),
                "offer differs: {differences:?}"
            );
            println!(
                "SIT_TRANSCRIPT {}",
                json!({"world": key, "point": sent["point"], "record": request})
            );
        }
        if let Ok(directory) = std::env::var("S34E_RECORD_DIR") {
            let frames = tagged(&lines, "SEATING_FRAME ");
            let path = std::path::Path::new(&directory).join(format!("posture-{key}.json"));
            std::fs::write(
                path,
                serde_json::to_string_pretty(&json!({"format": 1, "world": key, "frames": frames}))
                    .unwrap(),
            )
            .unwrap();
        }
        let drawn = tagged(&lines, "POSTURE ");
        assert!(!drawn[0]["seated"].is_null(), "observer drawn seated");
        assert!(drawn[1]["seated"].is_null(), "observer drawn standing");
        assert!(!drawn[2]["seated"].is_null(), "second sit drawn");
        let actor = requests[0]["request"]["actor"].as_str().unwrap();
        let tables = headless::Tables::read(std::path::Path::new(save.path()));
        let journal: Vec<_> = tables
            .journal
            .iter()
            .filter_map(|(_, bytes)| {
                let entry: JournalEntry = format::decode(bytes, "journal entry").unwrap();
                if let WorldInput::Dispatch { intent, .. } = entry.input {
                    let action = intent.action_type().as_str();
                    if intent.actor().raw().to_string() == actor && action != "walk-step" {
                        return Some(action.to_owned());
                    }
                }
                None
            })
            .collect();
        assert_eq!(
            journal,
            ["walk-to", "sit", "stand", "walk-to", "sit"],
            "persisted request order"
        );
        let recorded = facts(&save);
        assert!(recorded.iter().any(|(kind, _)| kind == "sat-down"));
        assert!(
            recorded
                .iter()
                .any(|(kind, value)| kind == "stood-up" && value["cause"] == "requested"),
            "requested stand: {recorded:?}"
        );
    }
}

#[tokio::test]
#[ignore = "requires Godot and two hosted seats; run at the S34-e milestone"]
async fn occupied_seat_is_sent_once_and_server_reason_is_shown() {
    for key in ["social-cafe", "market-town"] {
        let pack = format!("{}/../../worlds/{key}", env!("CARGO_MANIFEST_DIR"));
        let save = SaveDir::new(&format!("s34e-taken-{key}"));
        let world = World::start(&["server", &pack, "--save", save.path()], None).await;
        let mut player = support::Client::connect(world.address).await;
        let (actor, _) = player.join("visitor").await;
        let obs = serde_json::to_value(player.observation().await).unwrap();
        let place = obs["self_location"]["place"]["entity"].clone();
        let points = obs["entities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == place)
            .unwrap()["components"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["component_type"] == "sit-points")
            .unwrap()["payload"]
            .as_array()
            .unwrap();
        let point = &points[0];
        let x = point["at"]["x"].as_i64().unwrap() as i32;
        let y = point["at"]["y"].as_i64().unwrap() as i32;
        let place_id = serde_json::from_value(place).unwrap();
        // Setup uses real accepted walks. The 2D action under test still goes through a viewport click.
        player.walk_there(actor, place_id, Some((x - 700, y))).await;
        let mut other = support::Client::connect(world.address).await;
        let (occupant, _) = other.join("wanderer").await;
        other.observation().await;
        other.walk_there(occupant, place_id, Some((x, y))).await;
        other.submit_accepted(json!({"actor": occupant, "action_type": "sit", "target": null,
            "actor_location": null, "payload": {"action_type": "sit", "payload": {"point": point["key"]}}})).await;
        drop(player);
        let steps = json!([{"click_sit": point["key"]}, {"sleep": 1.0}]);
        let world_key = format!("--world-key={key}");
        let scratch = mineworld_test_support::scratch!(empty "s34e-taken-steps");
        let steps_path = godot2d::worlds::steps_file(&scratch, &steps);
        let steps_arg = format!("--steps={steps_path}");
        let drive = godot2d::Drive::start_seatless(
            world.address,
            &[
                "--drive=steps",
                &steps_arg,
                "--pick-seat=visitor",
                "--take-over",
                "--presentation=none",
                "--settings=none",
                &world_key,
            ],
        );
        let (status, lines) = drive.finish().await;
        assert!(status.success() && passed(&lines), "{key}: {lines:#?}");
        let requests = tagged(&lines, "REQUEST ");
        assert_eq!(
            requests.len(),
            1,
            "exactly one request, no walk or retry: {requests:?}"
        );
        assert_eq!(requests[0]["request"]["action_type"], "sit");
        let result = tagged(&lines, "RESULT ")
            .into_iter()
            .find(|r| r["token"] == requests[0]["token"])
            .unwrap();
        assert_eq!(result["result"]["rejected"], "target_unavailable");
        assert!(
            tagged(&lines, "TOAST ")
                .iter()
                .any(|t| t["kind"] == "rejected"
                    && t["action_type"] == "sit"
                    && t.to_string().contains("target_unavailable")),
            "server reason toasted: {lines:?}"
        );
    }
}
