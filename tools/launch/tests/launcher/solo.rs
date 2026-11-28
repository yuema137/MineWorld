//! `pr-launcher-solo.md` C1–C2: 2D plays Market Town as a visitor (3D and Both keep Social Café until
//! the Q-LS7 gate); one player alone is a `--solo` world, Both is not; a save played as Carol resumes with
//! the player as the visitor and Carol
//! town-driven, nothing reset; a fast-forward never survives a restart (`ARC-69` items 5 and 8).
//!
//! The oracle is the server, never the launcher's word: whether `POST /admin/clock` accepts 25×
//! (only a `--solo` world may), the scale `GET /admin/clock` reports after a restart, `welcome.took_over`,
//! and the wallet and holdings a seat's own observation discloses. The stub does what needs the
//! capability, which never leaves its memory; it records only status codes and scales.
use super::living::{http, join, until};
use super::*;

pub const PROBE: &str = "MINEWORLD_LAUNCH_SOLO_PROBE";

/// What the stub records, per client label: `<label>.solo` holds `<status of POST 25x> <scale before>`.
pub fn child(record: &Path, label: &str, args: &[String], phase: &str) {
    let value = |key: &str| {
        args.iter()
            .find_map(|v| v.strip_prefix(key))
            .expect("client argument")
    };
    let admin = std::env::var("MINEWORLD_ADMIN_TOKEN").expect("a played client has a capability");
    let address = value("--server=");
    let (status, clock) = http(address, Some(&admin), "/admin/clock", None);
    assert_eq!(status, 200, "GET /admin/clock");
    let before = clock["time_scale"].as_u64().expect("a scale");
    // "first": try a fast-forward and leave it set, so the next run shows whether it survived.
    let fast = if phase == "first" {
        http(
            address,
            Some(&admin),
            "/admin/clock",
            Some(json!({"time_scale": 25})),
        )
        .0
    } else {
        0
    };
    fs::write(
        record.join(format!("{label}.solo")),
        format!("{fast} {before}"),
    )
    .unwrap();
    if phase == "first" && value("--seat=") == "carol" {
        // The player's own state, changed by one of their own actions (a drink or a meal from what
        // they carry), so that a save reset to genesis cannot pass the comparison after the resume.
        let (seat, invite) = (value("--seat="), value("--invite="));
        let genesis = self_view(address, seat, invite).1;
        consume_own(address, seat, invite);
        let view = self_view(address, seat, invite);
        assert_ne!(genesis, view.1, "consuming changed the holdings");
        fs::write(record.join("carol.before"), view.1.to_string()).unwrap();
    }
}

/// Joins `seat`, consumes the first thing its own affordances offer to drink or eat, and leaves once
/// the server has accepted it.
fn consume_own(address: &str, seat: &str, invite: &str) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let (mut socket, welcome) = join(address, seat, invite).await;
        let view = until(&mut socket, |v| v["t"] == "observation").await;
        let offer = view["observation"]["affordances"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| {
                a["available"] == true && (a["action_type"] == "drink" || a["action_type"] == "eat")
            })
            .expect("the seat holds something to drink or eat")
            .clone();
        let kind = offer["action_type"].clone();
        socket.send(Message::Text(json!({"t":"submit", "token":"consume", "request":{
            "actor":welcome["observer"], "action_type":kind, "target":offer["target"],
            "payload":{"action_type":kind, "payload":offer["payload"]}, "actor_location":null
        }}).to_string().into())).await.unwrap();
        let result = until(&mut socket, |v| {
            v["t"] == "result" && v["token"] == "consume"
        })
        .await;
        assert!(
            result["result"]["accepted"].is_object(),
            "consuming accepted: {result}"
        );
        socket
            .send(Message::Text(json!({"t":"leave"}).to_string().into()))
            .await
            .unwrap();
    });
}

/// Joins `seat` and returns `welcome.took_over` and the wallet and holdings its own first observation
/// discloses, then leaves.
fn self_view(address: &str, seat: &str, invite: &str) -> (Value, Value) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let (mut socket, welcome) = join(address, seat, invite).await;
        let view = until(&mut socket, |v| v["t"] == "observation").await;
        let me = &welcome["observer"];
        let own = view["observation"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| &e["id"] == me)
            .expect("the observer sees itself")
            .clone();
        let pick = |kind: &str| {
            own["components"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["component_type"] == kind)
                .map(|c| c["payload"].clone())
                .unwrap_or_else(|| panic!("its own {kind} is disclosed to the seat"))
        };
        let owned = json!({"wallet": pick("wallet"), "holdings": pick("holdings")});
        socket
            .send(Message::Text(json!({"t":"leave"}).to_string().into()))
            .await
            .unwrap();
        (welcome["took_over"].clone(), owned)
    })
}

fn solo_record(bundle: &Bundle, label: &str) -> (u16, u64) {
    let text = fs::read_to_string(bundle.record().join(format!("{label}.solo")))
        .unwrap_or_else(|_| panic!("the {label} probe ran: {}", bundle.launch_logs()));
    let mut parts = text.split_whitespace().map(|v| v.parse::<u64>().unwrap());
    (
        u16::try_from(parts.next().unwrap()).unwrap(),
        parts.next().unwrap(),
    )
}

fn clear(bundle: &Bundle) {
    for entry in fs::read_dir(bundle.record()).unwrap() {
        let path = entry.unwrap().path();
        if path.file_name().and_then(|n| n.to_str()) != Some("carol.before") {
            fs::remove_file(path).unwrap();
        }
    }
}

fn instance(log: &str) -> String {
    log.lines()
        .find_map(|line| line.strip_prefix("[mineworld] world instance "))
        .expect("the server names its world instance")
        .to_owned()
}

pub fn market_town_by_default_solo_alone_and_a_carol_save_resumes_as_the_visitor() {
    let bundle = Bundle::new("launch-solo", &["market-town", "social-cafe"]);

    // A Social Café save made before the default change, kept byte-identical by it (LS-I4).
    bundle.overrides("[2d]\nworld = \"social-cafe\"\n");
    let cafe = bundle.launcher("2d", &[]).output().unwrap();
    assert!(cafe.status.success(), "{}", finished(&cafe));
    let cafe_save = bundle.user().join("saves/social-cafe");
    let cafe_bytes = files(&cafe_save);
    assert!(!cafe_bytes.is_empty(), "the Social Café save exists");
    let servers_before = bundle.logs("server");
    clear(&bundle);

    // Run 1: a save played as Carol, as every 2D save was before this change (LAUNCHER.md §4 opt-out).
    bundle.overrides("[2d]\nseat = \"carol\"\n");
    let first = bundle
        .launcher("2d", &[])
        .env(PROBE, "first")
        .output()
        .unwrap();
    assert!(first.status.success(), "{}", finished(&first));
    assert!(
        !bundle.record().join("2d.failed").exists(),
        "{}",
        bundle.launch_logs()
    );
    assert_eq!(bundle.record_of("2d").user("seat"), "carol");
    assert_eq!(
        solo_record(&bundle, "2d").0,
        200,
        "a 2D play run is a --solo world: 25x is permitted"
    );
    let first_log = new_log(&servers_before, &bundle.logs("server"));
    assert!(first_log.contains("[mineworld] created "), "{first_log}");

    // Run 2: the defaults. The same save resumes; the player is the visitor; Carol is town-driven.
    fs::remove_file(bundle.runtime().join("launch.toml")).unwrap();
    clear(&bundle);
    let servers = bundle.logs("server");
    let mut launcher = Background {
        bundle: &bundle,
        child: bundle
            .launcher("2d", &[])
            .env(STUB_WAIT, "1")
            .env(PROBE, "second")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    };
    let client = bundle.record_of("2d");
    assert_eq!(
        client.user("seat"),
        "visitor",
        "Q-LS6: 2D plays the visitor"
    );
    assert_eq!(client.user("world-key"), "market-town", "Q-LS1");
    let deadline = Instant::now() + PATIENCE;
    while !bundle.record().join("2d.solo").exists() {
        assert!(Instant::now() < deadline, "{}", bundle.launch_logs());
        std::thread::sleep(Duration::from_millis(50));
    }
    let (fast, scale) = solo_record(&bundle, "2d");
    assert_eq!(fast, 0);
    assert!(
        scale <= 24,
        "ARC-69 item 8: the 25x fast-forward did not survive the restart ({scale})"
    );
    let (took_over, after) = self_view(client.user("server"), "carol", client.user("invite"));
    assert_eq!(
        took_over, "hosted",
        "Carol was town-driven while the visitor played"
    );
    let before: Value =
        serde_json::from_str(&fs::read_to_string(bundle.record().join("carol.before")).unwrap())
            .unwrap();
    assert_eq!(before, after, "Carol's wallet and holdings were not reset");
    bundle.release("2d", 0);
    assert!(launcher.child.wait().unwrap().success());
    let second_log = new_log(&servers, &bundle.logs("server"));
    assert!(second_log.contains("[mineworld] resumed "), "{second_log}");
    assert_eq!(
        instance(&first_log),
        instance(&second_log),
        "the same world"
    );

    assert_eq!(
        files(&cafe_save),
        cafe_bytes,
        "the Social Café save is untouched"
    );

    // 3D alone is solo too; Both seats two players and is not. Both keep Social Café until the
    // Q-LS7 gate (`config.rs` WORLD_3D).
    for (mode, labels, expected) in [("3d", &["3d"][..], 200), ("both", &["2d", "3d"][..], 409)] {
        clear(&bundle);
        let run = bundle
            .launcher(mode, &[])
            .env(PROBE, "first")
            .output()
            .unwrap();
        assert!(run.status.success(), "{mode}: {}", finished(&run));
        for label in labels {
            assert_eq!(
                bundle.record_of(label).user("world-key"),
                "social-cafe",
                "{mode}"
            );
            assert_eq!(solo_record(&bundle, label).0, expected, "{mode} {label}");
        }
    }
}

/// Every file under `folder`, by relative path, with its bytes.
fn files(folder: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    let mut out = std::collections::BTreeMap::new();
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            for (inner, bytes) in files(&path) {
                out.insert(path.file_name().unwrap().into(), Vec::new());
                out.insert(PathBuf::from(path.file_name().unwrap()).join(inner), bytes);
            }
        } else {
            out.insert(path.file_name().unwrap().into(), fs::read(&path).unwrap());
        }
    }
    out
}
