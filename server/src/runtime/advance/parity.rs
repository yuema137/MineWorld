//! N2-I6 oracle parity (pr-nightlife N2-C3): the ordinary host, driven by an injected wall clock that ticks at
//! every due instant, is frozen as a fixture **before the advance loop existed**; the advance must reproduce it
//! — every consult, every answer, every fact byte in the save, and the instant reached.
//!
//! `MINEWORLD_WRITE_FIXTURES=1` rewrites a fixture from the ordinary host; never from the loop.

use std::path::PathBuf;

use super::town::{DAWN, Setup, night, ordinary, scratch};

/// Where a frozen oracle lives.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/runtime/advance/fixtures")
        .join(format!("{name}.txt"))
}

/// The frozen trace, rewritten from the ordinary host when asked.
fn frozen(name: &str, ordinary_trace: &[String]) -> Vec<String> {
    let path = fixture(name);
    if std::env::var_os("MINEWORLD_WRITE_FIXTURES").is_some() {
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("writable");
        std::fs::write(&path, ordinary_trace.join("\n") + "\n").expect("written");
    }
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} reads: {error}", path.display()))
        .lines()
        .map(str::to_owned)
        .collect()
}

/// The ordinary host still produces the frozen oracle: what the loop is compared with has not moved.
#[test]
fn the_ordinary_host_reproduces_the_frozen_oracle_night() {
    let mut whole = night(&scratch("oracle-whole"), Setup::default());
    let visited = ordinary(&mut whole);
    assert_eq!(visited.last().copied(), Some(DAWN), "it rests until dawn");
    let trace = whole.trace();
    assert_eq!(trace, frozen("night-whole", &trace));

    let mut nudged = night(
        &scratch("oracle-nudged"),
        Setup {
            nudge_at: Some(23 * 3_600 + 50 * 60),
            ..Setup::default()
        },
    );
    let visited = ordinary(&mut nudged);
    let woke = *visited.last().expect("visited");
    assert!(woke < DAWN, "the nudge wakes the sleeper before dawn: {woke}");
    let trace = nudged.trace();
    assert_eq!(trace, frozen("night-nudged", &trace));
}
