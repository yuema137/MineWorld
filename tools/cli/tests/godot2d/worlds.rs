//! Real worlds for the 2D client's interaction tests (step-13 §15.3 AC-I1 … AC-I3, AC-I6 … AC-I8):
//! entity ids resolved from the pack itself, a scripted player given a steps file, and the save's
//! fact log read back as the oracle — independent of the client under test.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::Path;
use std::process::ExitStatus;

use mineworld_contracts::{EventEnvelope, WorldTime};
use mineworld_persistence::format;
use serde_json::Value;

use super::Drive;
use crate::headless::Tables;
use crate::support::SaveDir;

/// `mineworld server <world> --agent alice --save <save>`: a hosted world as the operator runs it.
pub fn hosted(world: &str, save: &SaveDir) -> Vec<String> {
    ["server", world, "--agent", "alice", "--save", save.path()]
        .map(str::to_owned)
        .to_vec()
}

/// Every authored key of a World Pack and the identity string its entity was given at genesis.
pub fn ids(pack: &str) -> BTreeMap<String, String> {
    let read = mineworld_worldpack::WorldPack::read(Path::new(pack)).expect("the World Pack reads");
    let loaded = read.load(WorldTime::EPOCH).expect("it loads");
    loaded
        .ids()
        .iter()
        .map(|(key, id)| (key.as_str().to_owned(), id.raw().to_string()))
        .collect()
}

/// A save's facts in log order: each event type and its payload as JSON.
pub fn facts(save: &SaveDir) -> Vec<(String, Value)> {
    Tables::read(Path::new(save.path()))
        .facts
        .iter()
        .map(|(_, bytes)| {
            let fact: EventEnvelope = format::decode(bytes, "fact").expect("a fact");
            let payload: Value =
                serde_json::from_slice(fact.payload().payload()).expect("a JSON payload");
            (fact.event_type().as_str().to_owned(), payload)
        })
        .collect()
}

/// An identity as the client writes it, from any of the shapes a payload uses: a number, a string,
/// or a typed reference `{entity, entity_type}`.
pub fn id(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Object(fields) => fields.get("entity").map_or_else(String::new, id),
        _ => String::new(),
    }
}

/// Starts the scripted player on `steps` (written to a scratch file of its own), with `extra`
/// arguments; returns the running drive and the scratch guard that keeps the file alive.
pub fn start(
    address: SocketAddr,
    seat: &str,
    name: &str,
    steps: &Value,
    extra: &[&str],
) -> (Drive, mineworld_test_support::Scratch) {
    let dir = mineworld_test_support::scratch!(empty name);
    let path = dir.join("steps.json");
    std::fs::write(&path, serde_json::to_string_pretty(steps).expect("JSON"))
        .expect("steps written");
    let mut arguments = vec![
        "--drive=steps".to_owned(),
        format!("--steps={}", path.to_str().expect("a UTF-8 path")),
    ];
    arguments.extend(extra.iter().map(|a| (*a).to_owned()));
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    (Drive::start(address, seat, &borrowed), dir)
}

/// [`start`], then waits for the run to end.
pub async fn play(
    address: SocketAddr,
    seat: &str,
    name: &str,
    steps: &Value,
    extra: &[&str],
) -> (ExitStatus, Vec<String>) {
    let (drive, _dir) = start(address, seat, name, steps, extra);
    drive.finish().await
}

/// The `MENU` lines about `subject`, in order.
pub fn menus(lines: &[String], subject: &str) -> Vec<Value> {
    super::tagged(lines, "MENU ")
        .into_iter()
        .filter(|menu| menu["subject"] == subject)
        .collect()
}

/// Writes `steps` into `dir` and returns the file's path.
pub fn steps_file(dir: &Path, steps: &Value) -> String {
    let path = dir.join("steps.json");
    std::fs::write(&path, serde_json::to_string_pretty(steps).expect("JSON"))
        .expect("steps written");
    path.to_str().expect("a UTF-8 path").to_owned()
}

/// The index of the line where step `n` began (`STEP {step: n}`).
pub fn step_line(lines: &[String], n: usize) -> usize {
    lines
        .iter()
        .position(|line| {
            line.strip_prefix("STEP ")
                .and_then(|rest| serde_json::from_str::<Value>(rest).ok())
                .is_some_and(|step| step["step"] == n)
        })
        .unwrap_or_else(|| panic!("step {n} never began"))
}

/// The `STEP_DONE` report of step `n`.
pub fn step_done(lines: &[String], n: usize) -> Value {
    super::tagged(lines, "STEP_DONE ")
        .into_iter()
        .find(|done| done["step"] == n)
        .unwrap_or_else(|| panic!("step {n} never completed"))
}

/// The panel of `component` in a `PANELS`/`STEP_DONE` record, if shown.
pub fn panel(record: &Value, component: &str) -> Option<Value> {
    record["panels"]
        .as_array()?
        .iter()
        .find(|p| p["component"] == component)
        .cloned()
}

/// A directory copied whole, for a scratch World Pack that differs in one line.
pub fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the copy's directory");
    for entry in std::fs::read_dir(from).expect("a directory reads") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("a file copies");
        }
    }
}
