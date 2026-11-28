//! Played-launch integration: real hosting and protocol, with a renderer process stub.
//! The child consumes its admin capability in memory and never sends it to test output/files.
use super::*;
use std::io::{Read, Write};
use std::net::TcpStream;

pub const PROBE: &str = "MINEWORLD_LAUNCH_LIVING_PROBE";
const INHERITED: &str = "launcher-test-unrelated-parent-capability";
type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

pub fn child(record: &Path, label: &str, args: &[String], phase: &str) {
    let value = |key: &str| {
        args.iter()
            .find_map(|v| v.strip_prefix(key))
            .expect("client argument")
    };
    let admin = std::env::var("MINEWORLD_ADMIN_TOKEN").expect("played client has a capability");
    assert!(
        admin.len() == 32 && admin.bytes().all(|b| b.is_ascii_hexdigit()),
        "OS token shape"
    );
    assert!(
        admin != INHERITED && admin != value("--invite="),
        "fresh capability independent of invite and parent"
    );
    assert!(
        !args.iter().any(|v| v.contains(&admin)),
        "no secret in client argv"
    );
    assert!(
        !args.iter().any(|v| v == "--hosted-here"),
        "supervisor owns shutdown"
    );
    let address = value("--server=");
    assert_eq!(
        http(address, None, "/admin/clock", Some(json!({"time_scale":6}))).0,
        401
    );
    assert_eq!(
        http(
            address,
            Some(INHERITED),
            "/admin/clock",
            Some(json!({"time_scale":6}))
        )
        .0,
        401
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let (mut socket, welcome) = join(address, value("--seat="), value("--invite=")).await;
        assert_eq!(
            welcome["took_over"], "hosted",
            "played seat was controlled before joining"
        );
        let instance = welcome["world"]["instance"].as_str().unwrap();
        if phase == "resume" {
            assert_eq!(
                welcome["world"]["time_scale"], 6,
                "last day length persisted"
            );
            assert_eq!(
                fs::read_to_string(record.join("world-instance")).unwrap(),
                instance,
                "same saved world"
            );
        } else if label == "2d" {
            fs::write(record.join("world-instance"), instance).unwrap();
            assert_eq!(welcome["world"]["time_scale"], 12);
            for scale in [6, 12, 24] {
                assert_eq!(
                    http(
                        address,
                        Some(&admin),
                        "/admin/clock",
                        Some(json!({"time_scale":scale}))
                    )
                    .0,
                    200
                );
                let frame = until(&mut socket, |v| {
                    v["t"] == "clock" && v["time_scale"] == scale
                })
                .await;
                assert_eq!(frame["paused"], false);
            }
            assert_eq!(
                http(
                    address,
                    Some(&admin),
                    "/admin/clock",
                    Some(json!({"time_scale":48}))
                )
                .0,
                409,
                "both is not solo"
            );
            for paused in [true, false] {
                assert_eq!(
                    http(
                        address,
                        Some(&admin),
                        "/admin/clock",
                        Some(json!({"paused":paused}))
                    )
                    .0,
                    200
                );
                until(&mut socket, |v| v["t"] == "clock" && v["paused"] == paused).await;
            }
            eprintln!("living probe: clock controls confirmed; checking Bob reply");
            dialogue(&mut socket, &welcome).await;
            eprintln!("living probe: Bob reply confirmed; checking released Carol movement");
            movement_after_release(address, value("--invite=")).await;
            assert_eq!(
                http(
                    address,
                    Some(&admin),
                    "/admin/clock",
                    Some(json!({"time_scale":6}))
                )
                .0,
                200
            );
            until(&mut socket, |v| v["t"] == "clock" && v["time_scale"] == 6).await;
        }
        socket
            .send(Message::Text(json!({"t":"leave"}).to_string().into()))
            .await
            .unwrap();
        until(&mut socket, |v| {
            v["t"] == "closing" && v["reason"] == "left"
        })
        .await;
    });
    // The log folder is supplied as non-secret probe metadata. Inspect all process logs while the
    // capability is still available in memory; report no credential or response body on failure.
    let logs = record.parent().unwrap().join("user/logs");
    for entry in fs::read_dir(logs).unwrap() {
        let bytes = fs::read(entry.unwrap().path()).unwrap();
        assert!(
            !String::from_utf8_lossy(&bytes).contains(&admin),
            "secret leaked into a process log"
        );
    }
    fs::write(record.join(format!("{label}.verified")), "PASS").unwrap();
}

fn http(address: &str, admin: Option<&str>, path: &str, body: Option<Value>) -> (u16, Value) {
    let mut stream = TcpStream::connect(address).expect("local host HTTP");
    stream
        .set_read_timeout(Some(Duration::from_secs(8)))
        .unwrap();
    let method = if body.is_some() { "POST" } else { "GET" };
    let body = body.map_or_else(String::new, |v| v.to_string());
    let auth = admin.map_or_else(String::new, |v| format!("Authorization: Bearer {v}\r\n"));
    write!(stream, "{method} {path} HTTP/1.1\r\nHost: {address}\r\n{auth}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
    let mut answer = String::new();
    stream.read_to_string(&mut answer).unwrap();
    let (headers, body) = answer
        .split_once("\r\n\r\n")
        .expect("HTTP response boundary");
    let status = headers.split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, serde_json::from_str(body).unwrap_or(Value::Null))
}

async fn join(address: &str, seat: &str, invite: &str) -> (Socket, Value) {
    let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
        .await
        .unwrap();
    socket.send(Message::Text(json!({"t":"join", "protocol":2, "invite":invite, "nickname":"launcher-living-test", "seat":seat}).to_string().into())).await.unwrap();
    let welcome = until(&mut socket, |v| v["t"] == "welcome").await;
    (socket, welcome)
}

async fn until(socket: &mut Socket, predicate: impl Fn(&Value) -> bool) -> Value {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(40);
    loop {
        let next = tokio::time::timeout_at(deadline, socket.next())
            .await
            .expect("expected protocol evidence within 40 s");
        if let Some(Ok(Message::Text(text))) = next {
            let frame: Value = serde_json::from_str(&text).unwrap();
            if predicate(&frame) {
                return frame;
            }
            assert!(frame["t"] != "refused", "protocol request refused");
        } else if next.is_none() {
            panic!("connection ended before expected evidence");
        }
    }
}

async fn dialogue(socket: &mut Socket, welcome: &Value) {
    let view = until(socket, |v| v["t"] == "observation").await;
    let entities = view["observation"]["entities"].as_array().unwrap();
    // The fixture keeps Bob nearby at night. Resolve him from the named disclosure, not numeric ids.
    let bob = entities
        .iter()
        .find(|v| {
            v["components"].as_array().is_some_and(|cs| {
                cs.iter().any(|c| {
                    c["component_type"] == "display-name"
                        && c["payload"]["name"] == "Bob Achterberg"
                })
            })
        })
        .expect("Bob perceived");
    let target = bob["id"].clone();
    assert!(!target.is_null(), "observed Person identity");
    let utterance = "launcher living reply witness";
    socket.send(Message::Text(json!({"t":"submit", "token":"living-talk", "request":{
        "actor":welcome["observer"], "action_type":"talk", "target":target,
        "payload":{"action_type":"talk", "payload":{"utterance":utterance}}, "actor_location":null
    }}).to_string().into())).await.unwrap();
    let result = until(socket, |v| {
        v["t"] == "result" && v["token"] == "living-talk"
    })
    .await;
    assert!(
        result["result"]["accepted"].is_object(),
        "nearby talk accepted"
    );
    until(socket, |v| {
        let body = match v["t"].as_str() {
            Some("observation") => &v["observation"],
            Some("delta") => &v["delta"],
            _ => return false,
        };
        body["events"].as_array().is_some_and(|events| {
            events.iter().any(|event| {
                let said = &event["payload"]["payload"];
                event["event_type"] == "spoke"
                    && said["speaker"]["entity"] == target
                    && said["listener"]["entity"] == welcome["observer"]
                    && said["utterance"]
                        .as_str()
                        .is_some_and(|line| line.contains(&format!("You said \"{utterance}\"")))
            })
        })
    })
    .await;
}

async fn movement_after_release(address: &str, invite: &str) {
    let (mut socket, before) = join(address, "carol", invite).await;
    assert_eq!(before["took_over"], "hosted");
    let view = until(&mut socket, |v| v["t"] == "observation").await;
    let initial = view["observation"]["self_location"].clone();
    assert!(!initial.is_null());
    socket
        .send(Message::Text(json!({"t":"leave"}).to_string().into()))
        .await
        .unwrap();
    until(&mut socket, |v| v["t"] == "closing").await;
    tokio::time::sleep(Duration::from_secs(7)).await;
    let (mut resumed, after) = join(address, "carol", invite).await;
    assert_eq!(
        after["took_over"], "hosted",
        "leave restores the town controller"
    );
    assert_eq!(
        before["observer"], after["observer"],
        "same Person after controller switch"
    );
    let moved = until(&mut resumed, |v| v["t"] == "observation").await;
    assert_ne!(
        initial, moved["observation"]["self_location"],
        "non-Alice controller moved its Person after release"
    );
    resumed
        .send(Message::Text(json!({"t":"leave"}).to_string().into()))
        .await
        .unwrap();
}

pub fn played_town_and_private_clock_controls() {
    let bundle = Bundle::new("launch-living", &["social-cafe"]);
    bundle.overrides("[both]\nseat-2d = \"alice\"\nseat-3d = \"wanderer\"\n");
    // Deterministic fixture: Bob is available to talk; Carol has a due trip after release. No shipped
    // world is edited, and neither controller is replaced or told the expected action.
    let people = bundle.runtime().join("worlds/social-cafe/people");
    for (name, from, to) in [
        ("bob", "place: apartments", "place: cafe"),
        (
            "carol",
            "place: apartments, label: home",
            "place: cafe, label: home",
        ),
    ] {
        let path = people.join(format!("{name}.yaml"));
        fs::write(&path, fs::read_to_string(&path).unwrap().replace(from, to)).unwrap();
    }
    let mut launcher = Background {
        bundle: &bundle,
        child: bundle
            .launcher("both", &[])
            .env(STUB_WAIT, "1")
            .env(PROBE, "first")
            .env("MINEWORLD_ADMIN_TOKEN", INHERITED)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    };
    bundle.record_of("2d");
    bundle.record_of("3d");
    let deadline = Instant::now() + Duration::from_secs(120);
    while !bundle.record().join("2d.verified").exists()
        || !bundle.record().join("3d.verified").exists()
    {
        assert!(
            Instant::now() < deadline
                && !bundle.record().join("2d.failed").exists()
                && !bundle.record().join("3d.failed").exists(),
            "living probe failed or timed out; inspect retained client logs"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    bundle.release("2d", 0);
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        launcher.child.try_wait().unwrap().is_none(),
        "one remaining client keeps the server"
    );
    bundle.release("3d", 0);
    assert!(launcher.child.wait().unwrap().success());
    let old_logs = bundle.logs("server");
    for path in &old_logs {
        let text = fs::read_to_string(path).unwrap();
        assert!(text.contains("11 seat(s) driven in-server"));
        assert!(stopped_gracefully(&text));
    }
    for name in [
        "2d.args",
        "3d.args",
        "2d.verified",
        "3d.verified",
        "2d.release",
        "3d.release",
    ] {
        fs::remove_file(bundle.record().join(name)).unwrap();
    }
    let resumed = bundle
        .launcher("both", &[])
        .env(PROBE, "resume")
        .env("MINEWORLD_ADMIN_TOKEN", INHERITED)
        .output()
        .unwrap();
    assert!(resumed.status.success(), "launcher resume failed");
    assert!(
        bundle.record().join("2d.verified").exists()
            && bundle.record().join("3d.verified").exists(),
        "both clients confirmed saved clock and identity"
    );
    assert!(new_log(&old_logs, &bundle.logs("server")).contains("[mineworld] resumed "));
}
