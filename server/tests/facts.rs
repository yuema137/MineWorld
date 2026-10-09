//! Facts reaching observers through the server's two seams (step-12 §17, S11-C; `ARC-43`):
//! `observation.events`, the reliable `perceived` stream, `acted_through`, and cursors.
//!
//! The world is `support`'s two-system world; who learns of a fact is decided by a stub
//! [`EventPerception`] with a fixed table — `Public` to everybody, otherwise to the participants —
//! so that the claims here are about the server's queues and order, not about presence's rule
//! (which `systems/presence` and `tools/cli/tests` own).
//!
//! ```text
//! CA-4  observation.events is exact or accounted         in-process, a subscriber not reading
//! CA-5  an overflowing perceived stream is released lagged, and a resume loses nothing
//! CA-7  acted_through is per connection                  real sockets
//! CA-12 an ephemeral world serves cursors from the join on   real sockets
//! ```
//!
//! CA-4 and CA-5 are in-process because an operating system's socket buffers would absorb a client
//! that stops reading long before a bounded queue fills (`headless.rs` says the same).

mod support;

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use mineworld_contracts::{ActionId, EntityId, EventEnvelope, EventId, Visibility};
use mineworld_server::{
    ClosingReason, EventPerception, HistoryUnavailable, HostConfig, HostError, HostedWorld,
    JoinRequest, OfferedResume, PerceivedHistory, PerceivedJoin, PerceivesNothing, RefusalCode,
    ServerFrame, SessionId, Streamed, WorldHost, app,
};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use support::{ALICE, BOB, INVITE, key, speak_request, whisper_request};

const PATIENCE: Duration = Duration::from_secs(5);

/// Everything recorded since the host started, shared by the stub audience and the stub history.
type Log = Arc<Mutex<Vec<EventEnvelope>>>;

/// The fixed table: `Public` reaches everybody, anything else exactly its participants.
fn admitted(fact: &EventEnvelope, observer: EntityId) -> bool {
    match fact.visibility() {
        Visibility::Public => true,
        _ => fact.participants().contains(&observer),
    }
}

struct TableAudience(Log);

impl EventPerception for TableAudience {
    fn record(&mut self, fact: &EventEnvelope) {
        self.0.lock().expect("sound").push(fact.clone());
    }

    fn admits(&self, fact: &EventEnvelope, observer: EntityId) -> bool {
        admitted(fact, observer)
    }
}

/// The same table over the same log: a history a resume is served from.
struct TableHistory(Log);

impl PerceivedHistory for TableHistory {
    fn perceived(
        &self,
        observer: EntityId,
        since: Option<EventId>,
        through: EventId,
    ) -> Result<Vec<EventEnvelope>, HistoryUnavailable> {
        Ok(self
            .0
            .lock()
            .expect("sound")
            .iter()
            .filter(|fact| since.is_none_or(|since| fact.id() > since) && fact.id() <= through)
            .filter(|fact| admitted(fact, observer))
            .cloned()
            .collect())
    }
}

/// `support`'s world, perceiving no observation of its own (so `events` holds only what the server
/// fans out), with the table audience and, when asked, the table history.
fn world(log: &Log, history: bool) -> Result<HostedWorld, HostError> {
    let hosted = support::build()?
        .perceiving(PerceivesNothing)
        .perceiving_events(TableAudience(Arc::clone(log)));
    Ok(if history {
        hosted.with_history(TableHistory(Arc::clone(log)))
    } else {
        hosted
    })
}

async fn host(config: HostConfig, log: &Log, history: bool) -> WorldHost {
    let log = Arc::clone(log);
    WorldHost::spawn(config, move || world(&log, history))
        .await
        .expect("the world starts")
}

fn quick(event_backlog: usize, perceived_backlog: usize) -> HostConfig {
    HostConfig {
        event_backlog,
        perceived_backlog,
        ..support::brisk()
    }
}

fn speak(actor: EntityId, target: EntityId, words: &str) -> mineworld_contracts::ActionRequest {
    let wire: mineworld_contracts::ActionRequest<mineworld_server::WirePayload> =
        serde_json::from_value(speak_request(actor, target, words)).expect("a request");
    mineworld_server::protocol::into_kernel_request(&wire).expect("a kernel request")
}

/// Everything the stream carries for `window`: the sweep never stops sending observations, so a
/// reader stops on time rather than on silence.
async fn drain(
    stream: &mut tokio::sync::mpsc::Receiver<Streamed>,
    window: Duration,
) -> Vec<Streamed> {
    let deadline = tokio::time::Instant::now() + window;
    let mut items = Vec::new();
    while let Ok(Some(item)) = tokio::time::timeout_at(deadline, stream.recv()).await {
        items.push(item);
    }
    items
}

/// Somebody who speaks without holding a seat: an observer identity and nothing else.
struct Speaker(EntityId);

impl Speaker {
    const fn observer(&self) -> EntityId {
        self.0
    }
}

fn ids(events: &[mineworld_server::WireFact]) -> Vec<u64> {
    events
        .iter()
        .map(|event| event.envelope().id().raw())
        .collect()
}

/// CA-4: while a connection is not reading, its observations are dropped and its events queue
/// overflows; afterwards, every fact is either in exactly one observation's `events` or counted in
/// `events_dropped`, and the perceived stream — the reliable oracle — has every one of them, each
/// before the observation that carries it.
#[tokio::test]
async fn observation_events_are_exact_or_accounted() {
    exact_or_accounted(true).await;
}

/// The same without the perceived stream, where a full channel is met by the observation itself:
/// the oracle is then the log, judged by the same table.
#[tokio::test]
async fn observation_events_are_exact_or_accounted_without_the_perceived_stream() {
    exact_or_accounted(false).await;
}

async fn exact_or_accounted(perceiving: bool) {
    let log: Log = Arc::default();
    let host = host(quick(4, 4_096), &log, false).await;
    let mut bob = host
        .join_perceiving(
            JoinRequest::plain(key(BOB), SessionId::new(0)),
            perceiving.then_some(PerceivedJoin {
                since: Some(EventId::from_raw(4)),
            }),
        )
        .await
        .expect("bob is seated with a perceived stream from the head");
    // Alice is seated only to learn her observer; her stream is dropped (and reaped at the next
    // sweep), so that `events_dropped` — one counter for the world — counts Bob's queue alone.
    let seated = host.join(key(ALICE)).await.expect("alice is a seat");
    let alice = Speaker(seated.observer());
    drop(seated);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(host.status().await.expect("status").clients, 1);
    let before = host.status().await.expect("status").events_dropped;

    // Bob reads nothing while Alice speaks thirty times, with sweeps in between.
    for line in 0..30 {
        host.submit(
            alice.observer(),
            speak(alice.observer(), bob.observer(), &format!("line {line}")),
        )
        .await
        .expect("dispatched");
        if line % 5 == 0 {
            tokio::time::sleep(Duration::from_millis(60)).await;
        }
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
    let dropped = host.status().await.expect("status").events_dropped - before;

    // Now Bob reads until the stream is quiet.
    let (stream, _) = bob.streams();
    let mut reliable: Vec<u64> = Vec::new();
    let mut in_frames: Vec<u64> = Vec::new();
    let mut order_broken = Vec::new();
    for item in drain(stream, Duration::from_millis(600)).await {
        match item {
            Streamed::Facts { events, through } => {
                assert!(events.iter().all(|e| e.envelope().id() <= through));
                reliable.extend(ids(&events));
            }
            Streamed::Observation(perceived) => {
                for event in perceived.observation.events() {
                    let id = event.envelope().id().raw();
                    if perceiving && !reliable.contains(&id) {
                        order_broken.push(id);
                    }
                    in_frames.push(id);
                }
            }
        }
    }
    let spoken: Vec<u64> = log
        .lock()
        .expect("sound")
        .iter()
        .filter(|fact| admitted(fact, bob.observer()))
        .map(|fact| fact.id().raw())
        .collect();
    eprintln!(
        "{} facts, {} in observation frames, {dropped} dropped, {} on the perceived stream",
        spoken.len(),
        in_frames.len(),
        reliable.len()
    );

    if perceiving {
        assert_eq!(
            reliable, spoken,
            "the perceived stream has every fact, in order, once"
        );
    } else {
        assert!(
            reliable.is_empty(),
            "no perceived frame to a connection that did not ask"
        );
        assert!(in_frames.iter().all(|id| spoken.contains(id)));
    }
    let distinct: BTreeSet<u64> = in_frames.iter().copied().collect();
    assert_eq!(
        distinct.len(),
        in_frames.len(),
        "no fact in two frames' events"
    );
    assert!(
        dropped > 0,
        "the bounded queue overflowed, as the test intends"
    );
    assert_eq!(
        in_frames.len() as u64 + dropped,
        spoken.len() as u64,
        "every fact is delivered once in events or counted as dropped"
    );
    assert!(
        order_broken.is_empty(),
        "facts before the observations: {order_broken:?}"
    );
}

/// CA-5: a perceived stream that overflows its bound is released `lagged` with its seat held;
/// rejoining with its resume and its cursor serves exactly the facts it missed, and then the live
/// stream. Never a gap.
#[tokio::test]
async fn an_overflowing_perceived_stream_is_released_lagged_and_resumes_without_a_gap() {
    let log: Log = Arc::default();
    let host = host(quick(256, 8), &log, true).await;
    let cursor = EventId::from_raw(4);
    let mut bob = host
        .join_perceiving(
            JoinRequest::plain(key(BOB), SessionId::new(0)),
            Some(PerceivedJoin {
                since: Some(cursor),
            }),
        )
        .await
        .expect("seated");
    let alice = host.join(key(ALICE)).await.expect("seated");
    for line in 0..20 {
        host.submit(
            alice.observer(),
            speak(alice.observer(), bob.observer(), &format!("line {line}")),
        )
        .await
        .expect("dispatched");
    }
    let reason = tokio::time::timeout(PATIENCE, bob.released())
        .await
        .expect("released within patience")
        .expect("with a reason");
    assert_eq!(reason, ClosingReason::Lagged);

    // Rejoin with the resume (the seat was held) and the cursor the client still has.
    let resume = OfferedResume::new(bob.resume().reveal().to_owned());
    let mut again = host
        .join_perceiving(
            JoinRequest {
                seat: key(BOB),
                take_over: false,
                resume: Some(resume),
                session: SessionId::new(0),
            },
            Some(PerceivedJoin {
                since: Some(cursor),
            }),
        )
        .await
        .expect("the held seat is resumed");
    let start = again.perceived().expect("a perceived start").clone();
    let backfill = start.backfill.expect("facts were missed");
    assert_eq!(backfill.since, Some(cursor));
    let missed: Vec<u64> = backfill
        .history
        .perceived(again.observer(), backfill.since, backfill.through)
        .expect("readable")
        .iter()
        .map(|fact| fact.id().raw())
        .collect();

    host.submit(
        alice.observer(),
        speak(alice.observer(), again.observer(), "after the resume"),
    )
    .await
    .expect("dispatched");
    let (stream, _) = again.streams();
    let mut live = Vec::new();
    for item in drain(stream, Duration::from_millis(400)).await {
        if let Streamed::Facts { events, .. } = item {
            live.extend(ids(&events));
        }
    }
    let received: Vec<u64> = missed.iter().chain(&live).copied().collect();
    let spoken: Vec<u64> = log
        .lock()
        .expect("sound")
        .iter()
        .filter(|fact| admitted(fact, again.observer()))
        .map(|fact| fact.id().raw())
        .collect();
    assert_eq!(missed.len(), 20);
    assert_eq!(
        received, spoken,
        "the missed facts, then the live one: no gap, no duplicate"
    );
}

// ---------------------------------------------------------------------------------------------
// Over real sockets.
// ---------------------------------------------------------------------------------------------

async fn serve(log: &Log, history: bool) -> SocketAddr {
    let host = host(support::brisk(), log, history).await;
    let (listener, address) = app::bind("127.0.0.1:0".parse().expect("an address"))
        .await
        .expect("a port");
    tokio::spawn(async move {
        let _ = app::serve(listener, host, support::admission()).await;
    });
    address
}

struct Client {
    socket: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
}

impl Client {
    async fn connect(address: SocketAddr) -> Self {
        let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
            .await
            .expect("upgraded");
        Self { socket }
    }

    async fn send(&mut self, frame: &Value) {
        self.socket
            .send(Message::Text(frame.to_string().into()))
            .await
            .expect("sent");
    }

    async fn frame(&mut self) -> ServerFrame {
        loop {
            let message = tokio::time::timeout(PATIENCE, self.socket.next())
                .await
                .expect("a frame within patience")
                .expect("open")
                .expect("well formed");
            if let Message::Text(text) = message {
                return serde_json::from_str(&text).expect("a server frame");
            }
        }
    }

    /// Joins with extra fields; the first frame back.
    async fn join(&mut self, seat: &str, extra: Value) -> ServerFrame {
        let mut frame = json!({ "t": "join", "protocol": 2, "invite": INVITE, "nickname": "t",
                                "seat": seat });
        if let (Some(frame), Some(extra)) = (frame.as_object_mut(), extra.as_object()) {
            frame.extend(extra.clone());
        }
        self.send(&frame).await;
        self.frame().await
    }

    async fn submit(&mut self, token: &str, request: Value) -> ActionId {
        self.send(&json!({ "t": "submit", "token": token, "request": request }))
            .await;
        loop {
            if let ServerFrame::Result { action_id, .. } = self.frame().await {
                return action_id;
            }
        }
    }

    /// The `acted_through` of the next `count` observation frames.
    async fn acted(&mut self, count: usize) -> Vec<Option<ActionId>> {
        let mut seen = Vec::new();
        while seen.len() < count {
            if let ServerFrame::Observation { acted_through, .. } = self.frame().await {
                seen.push(acted_through);
            }
        }
        seen
    }
}

fn observer_of(welcome: &ServerFrame) -> (EntityId, String) {
    match welcome {
        ServerFrame::Welcome {
            observer, resume, ..
        } => (
            *observer,
            resume.as_ref().expect("a resume").reveal().to_owned(),
        ),
        other => panic!("expected a welcome, got {other:?}"),
    }
}

/// CA-7: `acted_through` is null before the first submit; after one answered `a`, the next
/// observation carries `a` and none later carries less; another connection's requests never show;
/// a new connection on the seat — a resume — starts at null.
#[tokio::test]
async fn acted_through_is_per_connection() {
    let log: Log = Arc::default();
    let address = serve(&log, false).await;
    let mut first = Client::connect(address).await;
    let mut other = Client::connect(address).await;
    let (alice, resume) = observer_of(&first.join(ALICE, json!({})).await);
    let (bob, _) = observer_of(&other.join(BOB, json!({})).await);

    assert!(
        first.acted(3).await.iter().all(Option::is_none),
        "null before any request"
    );
    let own = first.submit("a1", speak_request(alice, bob, "hello")).await;
    let theirs = other.submit("b1", whisper_request(bob, "mine")).await;
    let seen = first.acted(5).await;
    assert_eq!(
        seen[0],
        Some(own),
        "the first frame after the answer carries it"
    );
    assert!(
        seen.iter().all(|acted| *acted == Some(own)),
        "never less, never theirs: {seen:?}"
    );
    assert!(
        other
            .acted(5)
            .await
            .iter()
            .all(|acted| *acted == Some(theirs))
    );

    // The socket drops without leave; the seat is held; a new connection resumes it.
    drop(first);
    let mut resumed = Client::connect(address).await;
    let welcome = resumed.join(ALICE, json!({ "resume": resume })).await;
    assert!(
        matches!(welcome, ServerFrame::Welcome { .. }),
        "{welcome:?}"
    );
    assert!(
        resumed.acted(3).await.iter().all(Option::is_none),
        "a resumed seat is a new connection, whose requests these were not"
    );
}

/// CA-12, ephemeral half: a world with no history serves the perceived stream from the join on. A
/// cursor older than its head, or newer, is `cursor_unavailable` and grants nothing; the head itself
/// is welcomed, live only.
#[tokio::test]
async fn an_ephemeral_world_serves_cursors_from_the_join_on() {
    let log: Log = Arc::default();
    let address = serve(&log, false).await;
    let mut speaker = Client::connect(address).await;
    let (alice, _) = observer_of(&speaker.join(ALICE, json!({})).await);
    speaker
        .submit("s1", speak_request(alice, alice, "a fact after genesis"))
        .await;
    let head = log
        .lock()
        .expect("sound")
        .last()
        .expect("a fact")
        .id()
        .raw();

    let mut late = Client::connect(address).await;
    for since in [Value::Null, json!("1"), json!((head + 1_000).to_string())] {
        match late
            .join(BOB, json!({ "perceived": { "since": since } }))
            .await
        {
            ServerFrame::Refused { code, .. } => {
                assert_eq!(code, RefusalCode::CursorUnavailable, "since {since}");
            }
            other => panic!("since {since}: expected cursor_unavailable, got {other:?}"),
        }
    }
    let welcome = late
        .join(BOB, json!({ "perceived": { "since": head.to_string() } }))
        .await;
    let (bob, _) = observer_of(&welcome);
    speaker
        .submit("s2", speak_request(alice, bob, "live"))
        .await;
    let said_live = log.lock().expect("sound").last().expect("a fact").id();
    loop {
        match late.frame().await {
            ServerFrame::Perceived { through, events } => {
                let got: Vec<EventId> = events.iter().map(|e| e.envelope().id()).collect();
                assert_eq!(
                    got,
                    vec![said_live],
                    "live only: no backfill, nothing before"
                );
                assert_eq!(through, said_live);
                break;
            }
            ServerFrame::Observation { .. } => {}
            other => panic!("unexpected {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// CA-3's server half: judged when recorded, fact by fact, within one dispatch.
// ---------------------------------------------------------------------------------------------

mod rooms {
    //! A test-local world of two rooms, whose one action states two facts in one dispatch: a line
    //! heard in the room the actor is leaving, then the actor's arrival in the other room. Nothing
    //! between them can sweep, so only a fan-out that judges each fact as it is recorded gives the
    //! line to the person who said it on the way out (step-12 §17 CA-3, D-SC9).

    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    use mineworld_contracts::{
        Action, ActionIntent, ActionTypeId, EntityId, EntityKey, EntityType, Event, EventEnvelope,
        EventSchemaVersion, EventTypeId, PlaceId, SystemId, Visibility, WorldTime,
    };
    use mineworld_kernel::{
        Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
        WorldView,
    };
    use mineworld_server::{EventPerception, HostError, HostedWorld, PerceivesNothing, SeatRoster};
    use serde::{Deserialize, Serialize};

    pub struct Rooms;

    impl SystemIdentity for Rooms {
        const ID: SystemId = SystemId::from_static("rooms");
    }

    /// Say something in the room you are in; with `leaving`, then walk into the yard.
    #[derive(Serialize, Deserialize)]
    pub struct Say {
        pub leaving: bool,
    }

    impl Action for Say {
        const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("say");
        const OWNER: SystemId = Rooms::ID;
    }

    #[derive(Serialize, Deserialize)]
    pub struct Said;

    impl Event for Said {
        const EVENT_TYPE: EventTypeId = EventTypeId::from_static("said");
        const OWNER: SystemId = Rooms::ID;
        const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
    }

    #[derive(Serialize, Deserialize)]
    pub struct Went {
        pub person: EntityId,
    }

    impl Event for Went {
        const EVENT_TYPE: EventTypeId = EventTypeId::from_static("went");
        const OWNER: SystemId = Rooms::ID;
        const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
    }

    fn place(world_key: &str, read: &mineworld_kernel::WorldRead<'_>) -> PlaceId {
        let entity = read
            .resolve_key(&EntityKey::new(world_key).expect("a key"))
            .expect("the room exists");
        PlaceId::new(entity, EntityType::Place).expect("a place")
    }

    impl System for Rooms {
        const VERSION: SystemVersion = SystemVersion::new(1);

        fn declaration(&self) -> SystemDeclaration {
            SystemDeclaration::of::<Self>()
                .providing::<Say>()
                .emitting::<Said>()
                .emitting::<Went>()
        }

        fn resolve(
            &self,
            world: &mut WorldView<'_, Self>,
            intent: &ActionIntent,
        ) -> Result<Vec<Emission>, KernelError> {
            let unresolved = || KernelError::ActionNotResolvedBySystem {
                system: Self::ID,
                action_type: intent.action_type().clone(),
            };
            let bytes = intent
                .payload()
                .payload_for::<Say>()
                .map_err(|_| unresolved())?;
            let say: Say = serde_json::from_slice(bytes).map_err(|_| unresolved())?;
            let (hall, yard) = {
                let read = world.read();
                (place("hall", &read), place("yard", &read))
            };
            // The line names nobody: who hears it is the room's business alone.
            let mut facts = vec![
                Emission::new::<Said>(b"null".to_vec(), Visibility::Place(hall)).at_place(hall),
            ];
            if say.leaving {
                let went = serde_json::to_vec(&Went {
                    person: intent.actor(),
                })
                .expect("encodes");
                facts.push(
                    Emission::new::<Went>(went, Visibility::Place(yard))
                        .with_participants(vec![intent.actor()])
                        .at_place(yard),
                );
            }
            Ok(facts)
        }
    }

    /// Who is in which room, folded from `went` — every person starts in the hall.
    pub struct RoomAudience {
        rooms: BTreeMap<EntityId, PlaceId>,
        hall: PlaceId,
        pub log: Arc<Mutex<Vec<EventEnvelope>>>,
    }

    impl EventPerception for RoomAudience {
        fn record(&mut self, fact: &EventEnvelope) {
            self.log.lock().expect("sound").push(fact.clone());
            if let (Ok(bytes), Some(place)) = (fact.payload().payload_for::<Went>(), fact.place())
                && let Ok(went) = serde_json::from_slice::<Went>(bytes)
            {
                self.rooms.insert(went.person, place);
            }
        }

        fn admits(&self, fact: &EventEnvelope, observer: EntityId) -> bool {
            match fact.visibility() {
                Visibility::Place(place) => {
                    fact.participants().contains(&observer)
                        || self.rooms.get(&observer).copied().unwrap_or(self.hall) == *place
                }
                _ => false,
            }
        }
    }

    pub fn build(log: Arc<Mutex<Vec<EventEnvelope>>>) -> Result<HostedWorld, HostError> {
        let mut world = World::new();
        world.install(Rooms)?;
        for person in ["ann", "ben"] {
            world.create_entity(EntityKey::new(person).expect("a key"), EntityType::Person)?;
        }
        let mut hall = None;
        for room in ["hall", "yard"] {
            let id =
                world.create_entity(EntityKey::new(room).expect("a key"), EntityType::Place)?;
            hall.get_or_insert(PlaceId::new(id, EntityType::Place).expect("a place"));
        }
        world.genesis(WorldTime::EPOCH, Vec::new())?;
        let audience = RoomAudience {
            rooms: BTreeMap::new(),
            hall: hall.expect("the hall"),
            log,
        };
        Ok(HostedWorld::new(world)
            .seating(SeatRoster::new(
                ["ann", "ben"].map(|seat| EntityKey::new(seat).expect("a key")),
            ))
            .perceiving(PerceivesNothing)
            .perceiving_events(audience))
    }

    pub fn say(actor: EntityId, leaving: bool) -> mineworld_contracts::ActionRequest {
        let wire: mineworld_contracts::ActionRequest<mineworld_server::WirePayload> =
            serde_json::from_value(serde_json::json!({
                "actor": actor, "action_type": "say", "target": null,
                "payload": { "action_type": "say", "payload": { "leaving": leaving } },
                "actor_location": null,
            }))
            .expect("a request");
        mineworld_server::protocol::into_kernel_request(&wire).expect("a kernel request")
    }
}

/// CA-3's server half: Ben says a line in the hall and leaves for the yard in **one** dispatch; Ann
/// stays. Ben hears his own line (he was in the hall when it was recorded), and the yard arrival;
/// Ann hears the line but not the yard. A line Ann says afterwards reaches Ann and not Ben.
#[tokio::test]
async fn a_fact_is_judged_where_people_were_when_it_was_recorded() {
    let log: Log = Arc::default();
    let shared = Arc::clone(&log);
    let host = WorldHost::spawn(support::brisk(), move || rooms::build(shared))
        .await
        .expect("the world starts");
    let mut ann = host
        .join_perceiving(
            JoinRequest::plain(key("ann"), SessionId::new(0)),
            Some(PerceivedJoin { since: None }),
        )
        .await
        .expect("seated");
    let mut ben = host
        .join_perceiving(
            JoinRequest::plain(key("ben"), SessionId::new(0)),
            Some(PerceivedJoin { since: None }),
        )
        .await
        .expect("seated");

    host.submit(ben.observer(), rooms::say(ben.observer(), true))
        .await
        .expect("dispatched");
    host.submit(ann.observer(), rooms::say(ann.observer(), false))
        .await
        .expect("dispatched");
    let recorded: Vec<(u64, String)> = log
        .lock()
        .expect("sound")
        .iter()
        .map(|fact| (fact.id().raw(), fact.event_type().as_str().to_owned()))
        .collect();
    let [(line, _), (went, _), (later, _)] = recorded.as_slice() else {
        panic!("three facts: {recorded:?}");
    };

    let heard = |items: Vec<Streamed>| -> Vec<u64> {
        items
            .into_iter()
            .filter_map(|item| match item {
                Streamed::Facts { events, .. } => Some(ids(&events)),
                Streamed::Observation(_) => None,
            })
            .flatten()
            .collect()
    };
    let by_ben = heard(drain(ben.streams().0, Duration::from_millis(300)).await);
    let by_ann = heard(drain(ann.streams().0, Duration::from_millis(300)).await);
    assert_eq!(
        by_ben,
        vec![*line, *went],
        "the line said on his way out, then his arrival"
    );
    assert_eq!(
        by_ann,
        vec![*line, *later],
        "the hall's two lines, not the yard"
    );
}
