//! The people the corrected-memory scenario plays: plain protocol clients holding seats, and the `visitor` player
//! as a window (a protocol client) or as an outside agent through the MCP adapter (`mineworld_agent`).

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use mineworld_contracts::EntityId;
use mineworld_server::{ServerFrame, TookOver};
use serde_json::{Value, json};

use super::SETTLE;
use crate::support::cognition::{interpreter, repository};
use crate::support::{Client, INVITE, Server, join_frame, may_talk_to, own_history, talk};

/// A plain client holding a seat, with what its welcome said.
pub struct Held {
    pub client: Client,
    pub id: EntityId,
    pub took_over: TookOver,
}

pub async fn hold(server: &Server, seat: &str) -> Held {
    let mut client = Client::connect(server.address).await;
    match client.join_as(join_frame(seat)).await {
        ServerFrame::Welcome {
            observer,
            took_over,
            ..
        } => Held {
            client,
            id: observer,
            took_over,
        },
        other => panic!("{seat} is welcomed: {other:?}"),
    }
}

pub async fn leave(mut held: Held) {
    held.client.send(json!({ "t": "leave" })).await;
    held.client.disconnect().await;
}

/// Where a held person stands: their place and their position, from their own observation.
pub async fn whereabouts(held: &mut Held) -> (EntityId, (i32, i32)) {
    let observation = held.client.observation().await;
    let location = observation
        .self_location()
        .expect("a person knows where they are");
    let local = location.local().expect("a position in the place");
    (
        location.place().entity_id(),
        (local.x().value(), local.y().value()),
    )
}

/// The JSON of one of a person's own components, as their observation discloses it.
pub async fn own_component(held: &mut Held, component: &str) -> Value {
    let observation = held.client.observation().await;
    let me = observation
        .entity(observation.observer())
        .expect("I perceive myself");
    me.components()
        .iter()
        .find(|record| record.component_type().as_str() == component)
        .map_or(Value::Null, |record| record.payload().clone())
}

/// One player, on the `visitor` seat: a protocol client (a 2D or 3D window) or the MCP agent.
pub enum Player {
    Window { held: Box<Held>, alice: EntityId },
    Agent(Agent),
}

impl Player {
    pub async fn say(&mut self, words: &str) {
        match self {
            Self::Window { held, alice } => {
                let alice = *alice;
                held.client
                    .observation_where("talk to Alice available", |seen| may_talk_to(seen, alice))
                    .await;
                tokio::time::sleep(SETTLE).await;
                let request = talk(held.id, alice, words);
                held.client.submit_accepted(request).await;
            }
            Self::Agent(agent) => {
                let said = agent.ask(&json!({ "say": words }));
                assert_eq!(said["said"], "accepted", "the agent's line is accepted");
            }
        }
    }

    /// Every line Alice has said to this player, as the player was told.
    pub async fn heard(&mut self) -> Vec<String> {
        match self {
            Self::Window { held, alice } => {
                let observation = held.client.observation().await;
                own_history(&observation).map_or_else(Vec::new, |history| {
                    history
                        .heard()
                        .iter()
                        .filter(|line| line.speaker().entity_id() == *alice)
                        .map(|line| line.utterance().as_str().to_owned())
                        .collect()
                })
            }
            Self::Agent(agent) => agent.ask(&json!({ "heard": true }))["heard"]
                .as_array()
                .expect("lines")
                .iter()
                .map(|line| line.as_str().expect("words").to_owned())
                .collect(),
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Window { .. } => "protocol-client",
            Self::Agent(_) => "mcp-agent",
        }
    }
}

/// `clients/agent/tests/correction_mcp_player.py`: the adapter over stdio, driven one JSON line at a time.
pub struct Agent {
    pub child: Child,
    pub stdin: ChildStdin,
    pub stdout: BufReader<ChildStdout>,
    pub ready: Value,
}

impl Agent {
    pub fn start(server: &Server, root: &Path) -> Self {
        let mut child = Command::new(interpreter())
            .arg(repository().join("clients/agent/tests/correction_mcp_player.py"))
            .args(["--server", &format!("ws://{}/ws", server.address)])
            .arg("--state")
            .arg(root.join("agent-state"))
            .arg("--log")
            .arg(root.join("agent-adapter.log"))
            .env("MINEWORLD_INVITE", INVITE)
            .env("PYTHONUNBUFFERED", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("the agent driver starts");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));
        let mut agent = Self {
            child,
            stdin,
            stdout,
            ready: Value::Null,
        };
        agent.ready = agent.answer()["ready"].clone();
        assert!(agent.ready.is_object(), "the agent joined: {}", agent.ready);
        agent
    }

    pub fn answer(&mut self) -> Value {
        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .expect("the driver answers");
        serde_json::from_str(&line).unwrap_or_else(|_| panic!("a JSON answer: {line:?}"))
    }

    pub fn ask(&mut self, command: &Value) -> Value {
        writeln!(self.stdin, "{command}").expect("the command is sent");
        self.answer()
    }

    pub fn leave(mut self) {
        assert_eq!(self.ask(&json!({ "leave": true }))["left"], true);
        let status = self.child.wait().expect("the driver ends");
        assert!(
            status.success(),
            "the agent driver exited cleanly: {status}"
        );
    }
}

/// Feeds the held person from what she carries, through meals the world offers her, until none is offered (at most
/// four): a person held still from genesis to the gate's opening is otherwise famished there, and her rule
/// fallback eating during the conversation changes a need word and asks a `feeling` consult the §6 scenario never
/// meant (red-team #565). Returns the action types eaten or drunk.
pub async fn feed(held: &mut Held) -> Vec<String> {
    let mut meals = Vec::new();
    for _ in 0..4 {
        let observation = held.client.observation().await;
        let Some(meal) = observation
            .affordances()
            .iter()
            .find(|offer| {
                matches!(offer.action_type().as_str(), "eat" | "drink")
                    && offer.is_available()
                    && offer.payload().is_some()
            })
            .cloned()
        else {
            break;
        };
        let kind = meal.action_type().as_str().to_owned();
        let request = json!({
            "actor": held.id,
            "action_type": kind,
            "target": meal.target(),
            "payload": { "action_type": kind, "payload": meal.payload() },
            "actor_location": null,
        });
        held.client.submit_accepted(request).await;
        meals.push(kind);
    }
    meals
}
