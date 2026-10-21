//! Many players, a living town (step-13 S12 PR 13e): real 2D clients against a real hosted server,
//! AC-13e-1 … AC-13e-8 of `.structured-coding/plans/mvp0/pr-13e-many-players-2d.md` §6.
//!
//! Every test starts Godot, so every test is `#[ignore]`d: `cargo test` reports them as ignored,
//! never as passed, on a machine without Godot (a test not run is not a pass). Run them with
//!
//! ```text
//! cargo test -p mineworld-cli --test client_2d_many -- --ignored --test-threads=1
//! ```
//!
//! Oracles are the server's: `/status`, its welcome and closing codes as the client printed them, the
//! identity another client's observation listed, and `mineworld perceived` — never the client's own
//! verdict alone. Every criterion runs on a world besides market-town (lane charter).

mod godot2d;
mod support;

use godot2d::{Drive, MARKET_TOWN, World, evidence, passed, tagged};
use serde_json::Value;
use support::SaveDir;

const SOCIAL_CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");
const LAKESIDE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/lakeside");

/// `mineworld server <world> --save <save>` plus `extra`, as owned arguments.
fn hosted(world: &str, save: &SaveDir, extra: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = ["server", world, "--save", save.path()]
        .map(str::to_owned)
        .to_vec();
    out.extend(extra.iter().map(|a| (*a).to_owned()));
    out
}

fn args(owned: &[String]) -> Vec<&str> {
    owned.iter().map(String::as_str).collect()
}

/// The World Pack's directory name, which the client is told for its far field.
fn world_key(world: &str) -> String {
    format!(
        "--world-key={}",
        std::path::Path::new(world)
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a world directory")
    )
}

/// The welcome evidence lines, in order.
fn welcomes(lines: &[String]) -> Vec<Value> {
    evidence(lines, "welcome")
}

/// AC-13e-5 — the client's socket closes under it mid-session: it is welcomed again `held`, as the
/// same observer, within the server's default 30 s hold, and no `seat_occupied` is ever answered.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_many -- --ignored"]
async fn a_blink_resumes_the_same_seat() {
    for (world, seat) in [(MARKET_TOWN, "visitor"), (SOCIAL_CAFE, "wanderer")] {
        let save = SaveDir::new("2d-many-blink");
        let mut server = World::start(&args(&hosted(world, &save, &[])), None).await;
        let key = world_key(world);
        let (status, lines) = Drive::start(server.address, seat, &["--drive=blink", &key])
            .finish()
            .await;
        server.kill();
        assert!(status.success() && passed(&lines), "{world}: the drive passes");
        let welcomed = welcomes(&lines);
        assert_eq!(welcomed.len(), 2, "{world}: welcomed twice: {welcomed:?}");
        assert_eq!(welcomed[0]["observer"], welcomed[1]["observer"], "{world}: the same Person");
        assert_eq!(welcomed[1]["took_over"], "held", "{world}: {:?}", welcomed[1]);
        assert!(
            !tagged(&lines, "REFUSED ")
                .iter()
                .any(|r| r["code"] == "seat_occupied"),
            "{world}: never seat_occupied"
        );
        let back = &evidence(&lines, "blink_back")[0];
        assert!(
            back["after_ms"].as_u64().expect("after_ms") <= 30_000 + 5_000,
            "{world}: within the hold: {back}"
        );
    }
}
