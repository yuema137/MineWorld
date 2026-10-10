//! CP-C1 (CA-8) and CA-9 over real frames: measure before adopting (step-12 §17 SD-C10, `DEP-15`).
//!
//! A hosted market town with four sessions — visitor, wanderer, and two seats taken over from their
//! in-server controllers — is recorded for 60 wall seconds. Every frame the server sends is a whole
//! observation, so on the same frames this computes, per client per second:
//!
//! ```text
//! whole       the observation frames as sent
//! typed       frame 1 whole, then a `delta` frame of the server's own diff (PROTOCOL.md §5.3)
//! json-patch  frame 1 whole, then a `delta` frame carrying an RFC 6902 patch (json-patch 4.2)
//! ```
//!
//! and SD-C10's frozen rule decides: typed if ≤ ½ whole and json-patch is not within 10 % of it;
//! json-patch if within 10 % of typed and ≤ ½ whole; whole observations only otherwise. On the same
//! recorded pairs it checks CA-9: apply(prev, diff(prev, next)) equals next in canonical order.
//!
//! `#[ignore]`: a measurement, run explicitly and recorded in step-12 §17.12, not a guard.
//!
//! ```text
//! cargo test -p mineworld-cli --test deltas -- --ignored --nocapture
//! ```

mod support;

use std::time::{Duration, Instant};

use mineworld_server::protocol::delta::{apply, canonical, diff};
use mineworld_server::{ServerFrame, WireObservation};
use serde_json::json;
use support::{Client, MARKET_PACK, Server};

const WINDOW: Duration = Duration::from_secs(60);

/// One client's recorded observation frames, in order: the frame's text length and the frame.
struct Recorded {
    seat: &'static str,
    frames: Vec<(usize, u64, Option<u64>, WireObservation)>,
}

#[tokio::test]
#[ignore = "CP-C1: a 60 s measurement whose numbers decide DEP-15; run explicitly"]
async fn cp_c1_whole_typed_and_json_patch_bytes_on_a_hosted_town() {
    // Every frame whole, so that all three encodings are computed from the same frames (CA-8).
    let server = Server::start(&["server", MARKET_PACK, "--town", "--keyframe-every", "1"]).await;
    let mut clients = Vec::new();
    for seat in ["visitor", "wanderer", "alice", "bob"] {
        let mut client = Client::connect(server.address).await;
        client.join(seat).await;
        clients.push((seat, client));
    }

    // Read every client in turn until the window closes; text lengths are of the frames as sent.
    let deadline = Instant::now() + WINDOW;
    let mut recorded: Vec<Recorded> = clients
        .iter()
        .map(|(seat, _)| Recorded {
            seat,
            frames: Vec::new(),
        })
        .collect();
    while Instant::now() < deadline {
        for (index, (_, client)) in clients.iter_mut().enumerate() {
            let text = client.text().await;
            if let Ok(ServerFrame::Observation {
                seq,
                revision,
                observation,
                ..
            }) = serde_json::from_str::<ServerFrame>(&text)
            {
                recorded[index].frames.push((
                    text.len(),
                    seq,
                    revision.map(|r| r.raw()),
                    observation,
                ));
            }
        }
    }

    let seconds = WINDOW.as_secs_f64();
    let (mut whole_all, mut typed_all, mut patch_all) = (0.0, 0.0, 0.0);
    let mut pairs = 0usize;
    let mut mismatches = Vec::new();
    for client in &recorded {
        let mut whole = 0usize;
        let mut typed = 0usize;
        let mut patch = 0usize;
        let mut previous: Option<&WireObservation> = None;
        for (length, seq, revision, observation) in &client.frames {
            whole += length;
            match previous {
                None => {
                    typed += length;
                    patch += length;
                }
                Some(prev) => {
                    let delta = diff(prev, observation);
                    let frame = json!({ "t": "delta", "seq": seq, "base": seq - 1,
                                        "revision": revision, "acted_through": null,
                                        "delta": delta });
                    typed += frame.to_string().len();
                    let ops = json_patch::diff(
                        &serde_json::to_value(prev).expect("JSON"),
                        &serde_json::to_value(observation).expect("JSON"),
                    );
                    let frame = json!({ "t": "delta", "seq": seq, "base": seq - 1,
                                        "revision": revision, "acted_through": null,
                                        "patch": ops });
                    patch += frame.to_string().len();
                    pairs += 1;
                    let rebuilt = apply(prev, &delta).expect("applies");
                    if rebuilt != canonical(observation.clone()) {
                        mismatches.push((client.seat, *seq));
                    }
                }
            }
            previous = Some(observation);
        }
        #[allow(clippy::cast_precision_loss)]
        let rate = |bytes: usize| bytes as f64 / seconds;
        eprintln!(
            "CP-C1 {:<9} {:>4} frames  whole {:>9.0} B/s  typed {:>8.0} B/s  json-patch {:>8.0} B/s",
            client.seat,
            client.frames.len(),
            rate(whole),
            rate(typed),
            rate(patch),
        );
        whole_all += rate(whole);
        typed_all += rate(typed);
        patch_all += rate(patch);
    }
    let clients = recorded.len() as f64;
    let (whole, typed, patch) = (
        whole_all / clients,
        typed_all / clients,
        patch_all / clients,
    );
    // SD-C10's frozen rule, literally.
    let within_ten_percent = (patch - typed).abs() <= typed * 0.10;
    let outcome = if typed <= whole / 2.0 && !within_ten_percent {
        "typed"
    } else if within_ten_percent && patch <= whole / 2.0 {
        "json-patch"
    } else {
        "keyframes only"
    };
    eprintln!(
        "CP-C1 mean per client per second: whole {whole:.0} B/s, typed {typed:.0} B/s ({:.1} %), \
         json-patch {patch:.0} B/s ({:.1} %); {pairs} consecutive pairs; outcome: {outcome}",
        100.0 * typed / whole,
        100.0 * patch / whole,
    );
    assert!(
        pairs >= 2_000,
        "CA-9 needs at least 2 000 recorded pairs, had {pairs}"
    );
    assert!(
        mismatches.is_empty(),
        "CA-9: reconstruction differs at {mismatches:?}"
    );
}

/// The frames one connection is sent for `window`, as sent, decoded without applying anything:
/// (seq, base — `None` for a whole observation, whether it applied cleanly to the frame held).
async fn raw_stream(client: &mut Client, window: Duration) -> Vec<(u64, Option<u64>, bool)> {
    let deadline = Instant::now() + window;
    let mut held: Option<(u64, WireObservation)> = None;
    let mut seen = Vec::new();
    while Instant::now() < deadline {
        match serde_json::from_str::<ServerFrame>(&client.text().await).expect("a frame") {
            ServerFrame::Observation {
                seq, observation, ..
            } => {
                held = Some((seq, observation));
                seen.push((seq, None, true));
            }
            ServerFrame::Delta {
                seq, base, delta, ..
            } => {
                let applied = held
                    .as_ref()
                    .filter(|(held_seq, _)| *held_seq == base)
                    .and_then(|(_, observation)| apply(observation, &delta).ok());
                let clean = applied.is_some();
                if let Some(next) = applied {
                    held = Some((seq, next));
                }
                seen.push((seq, Some(base), clean));
            }
            _ => {}
        }
    }
    seen
}

/// CA-10: deltas on the wire, through the binary with the default `--keyframe-every 50`. Frame 1 is
/// whole; every delta's base is the frame before it and applies cleanly; frames 50, 100, … are whole;
/// the first frame after a perceived backfill and after a resume is whole. 60 wall seconds.
#[tokio::test]
async fn deltas_on_the_wire_keyframes_and_clean_application() {
    let save = mineworld_test_support::scratch!("deltas-on-the-wire");
    let path = save.to_str().expect("a printable path").to_owned();
    let server = Server::start(&[
        "server",
        MARKET_PACK,
        "--town",
        "--save",
        &path,
        "--hold",
        "10",
    ])
    .await;
    let mut first = Client::connect(server.address).await;
    let welcome = first
        .join_as(
            json!({ "t": "join", "protocol": 2, "invite": support::INVITE,
                         "nickname": "deltas", "seat": "wanderer",
                         "perceived": { "since": null } }),
        )
        .await;
    let ServerFrame::Welcome { resume, .. } = welcome else {
        panic!("welcomed: {welcome:?}");
    };
    let resume = resume.expect("a resume").reveal().to_owned();
    let mut seen = raw_stream(&mut first, Duration::from_secs(50)).await;
    first.disconnect().await;

    let mut second = Client::connect(server.address).await;
    let welcome = second
        .join_as(
            json!({ "t": "join", "protocol": 2, "invite": support::INVITE,
                         "nickname": "deltas", "seat": "wanderer", "resume": resume }),
        )
        .await;
    assert!(
        matches!(welcome, ServerFrame::Welcome { .. }),
        "{welcome:?}"
    );
    let after_resume = raw_stream(&mut second, Duration::from_secs(10)).await;

    let deltas = seen.iter().filter(|(_, base, _)| base.is_some()).count();
    eprintln!(
        "CA-10: {} frames before the drop ({deltas} deltas), {} after the resume",
        seen.len(),
        after_resume.len()
    );
    assert_eq!(
        seen[0],
        (1, None, true),
        "the first frame, after the backfill, is whole"
    );
    assert_eq!(
        after_resume[0].1, None,
        "the first frame after a resume is whole"
    );
    seen.extend(
        after_resume
            .iter()
            .map(|(seq, base, clean)| (*seq + 1_000_000, base.map(|b| b + 1_000_000), *clean)),
    );
    for window in seen.windows(2) {
        let ((previous, _, _), (seq, base, clean)) = (window[0], window[1]);
        if let Some(base) = base {
            assert_eq!(
                base, previous,
                "frame {seq}: a delta's base is the frame before it"
            );
            assert!(
                clean,
                "frame {seq}: applies with no base mismatch and no unknown remove"
            );
        }
    }
    for (seq, base, _) in &seen {
        if *seq < 1_000_000 && seq % 50 == 0 {
            assert_eq!(*base, None, "frame {seq} is a keyframe");
        }
    }
    assert!(deltas > 300, "deltas were sent: {deltas}");
}
