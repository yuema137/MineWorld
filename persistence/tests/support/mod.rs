//! A small world that exercises every kind of state a save must carry, and a scratch directory.
//!
//! ```text
//! ledger   owns `tally`; `note` records a fact about whom it names, relates the two, and defers a
//!          reminder; `brew` starts a process that ends five minutes later; a `note` with empty text
//!          is rejected, and one whose text is "fault" makes the system break its own contract
//! echo     owns `heard`; reduces ledger's `noted` facts — a second system's state
//! ```
//!
//! Every component row, edge, deferral, process and counter is produced by the kernel's own
//! pipeline; nothing here writes state any other way.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionTypeId, EntityId, EntityKey, EntityType,
    EntityTypeSet, Event, EventEnvelope, EventSchemaVersion, EventTypeId, ProcessTypeId, Rejection,
    RelationTypeDeclaration, RelationTypeId, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, Process, ProcessKind, ProcessStart, System,
    SystemDeclaration, SystemIdentity, SystemVersion, World, WorldRead, WorldView, owned_component,
};
use serde::{Deserialize, Serialize};

pub fn encode<T: Serialize + ?Sized>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("encodes")
}

pub fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

// ---------------------------------------------------------------------------------------------
// ledger and echo
// ---------------------------------------------------------------------------------------------

pub struct Ledger;

impl SystemIdentity for Ledger {
    const ID: SystemId = SystemId::from_static("ledger");
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tally {
    pub count: u32,
    pub last: String,
}

owned_component! {
    component = Tally,
    owner = Ledger,
    component_type = "tally",
    schema_version = 1,
}

fn noted_by() -> RelationTypeId {
    RelationTypeId::new("noted-by").expect("a legal name")
}

fn noted_by_declaration() -> RelationTypeDeclaration {
    RelationTypeDeclaration::directed(
        noted_by(),
        Ledger::ID,
        EntityTypeSet::new([EntityType::Person]).expect("non-empty"),
        EntityTypeSet::new([EntityType::Person]).expect("non-empty"),
    )
}

#[derive(Serialize, Deserialize)]
pub struct Note {
    pub text: String,
}

impl Action for Note {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("note");
    const OWNER: SystemId = Ledger::ID;
}

#[derive(Serialize, Deserialize)]
pub struct Brew;

impl Action for Brew {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("brew");
    const OWNER: SystemId = Ledger::ID;
}

#[derive(Serialize, Deserialize)]
pub struct Noted {
    pub text: String,
}

impl Event for Noted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("noted");
    const OWNER: SystemId = Ledger::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Serialize, Deserialize)]
pub struct Reminded {
    pub text: String,
}

impl Event for Reminded {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("reminded");
    const OWNER: SystemId = Ledger::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Serialize, Deserialize)]
pub struct Brewed;

impl Event for Brewed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("brewed");
    const OWNER: SystemId = Ledger::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

pub struct Brewing;

impl ProcessKind for Brewing {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("brewing");
    type Owner = Ledger;
}

pub const BREW: i64 = 300;
pub const REMINDER: i64 = 100;

impl System for Ledger {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Tally>()
            .providing::<Note>()
            .providing::<Brew>()
            .emitting::<Noted>()
            .emitting::<Reminded>()
            .emitting::<Brewed>()
            .subscribing_to::<Noted>()
            .subscribing_to::<Reminded>()
            .subscribing_to::<Brewed>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Tally>()?;
        tables.relation(noted_by_declaration())
    }

    fn validate(&self, _: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        if *intent.action_type() == Note::ACTION_TYPE {
            let note: Note = serde_json::from_slice(intent.payload().payload()).expect("a note");
            if note.text.is_empty() {
                return Err(Rejection::PreconditionFailed);
            }
        }
        Ok(())
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let subject = intent.target().unwrap_or(intent.actor());
        if *intent.action_type() == Brew::ACTION_TYPE {
            world.start_process(
                ProcessStart::<Brewing>::new(Vec::new())
                    .with_participants(vec![subject])
                    .ending_at(t(world.at().seconds() + BREW)),
            )?;
            return Ok(Vec::new());
        }
        let note: Note = serde_json::from_slice(intent.payload().payload()).expect("a note");
        if note.text == "fault" {
            // A system breaking its own contract: it provides `note` and refuses to resolve it.
            return Err(KernelError::ActionNotResolvedBySystem {
                system: Self::ID,
                action_type: Note::ACTION_TYPE,
            });
        }
        if let Some(target) = intent.target() {
            world.relate(&noted_by(), target, intent.actor())?;
        }
        world.defer(
            t(world.at().seconds() + REMINDER),
            Emission::new::<Reminded>(
                encode(&Reminded {
                    text: note.text.clone(),
                }),
                Visibility::Public,
            )
            .about(vec![subject]),
        )?;
        Ok(vec![
            Emission::new::<Noted>(encode(&Noted { text: note.text }), Visibility::Public)
                .about(vec![subject]),
        ])
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let subject = event.subjects()[0];
        let mut tally = world
            .read()
            .component::<Tally>(subject)
            .cloned()
            .unwrap_or_default();
        tally.count += 1;
        tally.last = event.event_type().to_string();
        world.insert(subject, tally)?;
        Ok(Vec::new())
    }

    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        world.end_process::<Brewing>(process.id())?;
        Ok(vec![
            Emission::new::<Brewed>(Vec::new(), Visibility::Public)
                .about(process.participants().to_vec()),
        ])
    }
}

pub struct Echo;

impl SystemIdentity for Echo {
    const ID: SystemId = SystemId::from_static("echo");
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heard {
    pub times: u32,
}

owned_component! {
    component = Heard,
    owner = Echo,
    component_type = "heard",
    schema_version = 1,
}

impl System for Echo {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Heard>()
            .subscribing_to::<Noted>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Heard>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let subject = event.subjects()[0];
        let mut heard = world
            .read()
            .component::<Heard>(subject)
            .cloned()
            .unwrap_or_default();
        heard.times += 1;
        world.insert(subject, heard)?;
        Ok(Vec::new())
    }
}

/// The systems installed and nothing else: what a restart starts from.
pub fn composed() -> World {
    let mut world = World::new();
    world.install(Ledger).expect("ledger installs");
    world.install(Echo).expect("echo installs");
    world
}

pub const PEOPLE: [&str; 4] = ["ada", "bo", "cy", "di"];

/// Composed, with its people created — assembled and not yet begun.
pub fn assembled() -> (World, Vec<EntityId>) {
    let mut world = composed();
    let people = PEOPLE
        .iter()
        .map(|key| {
            world
                .create_entity(EntityKey::new(*key).expect("a key"), EntityType::Person)
                .expect("created")
        })
        .collect();
    (world, people)
}

/// The genesis facts: every person begins with one note about themselves.
pub fn genesis_facts(people: &[EntityId]) -> Vec<Emission> {
    people
        .iter()
        .map(|person| {
            Emission::new::<Noted>(
                encode(&Noted {
                    text: "born".to_owned(),
                }),
                Visibility::Public,
            )
            .about(vec![*person])
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// the script
// ---------------------------------------------------------------------------------------------

/// One step of a deterministic script: at an instant, somebody notes something about somebody, or
/// brews.
#[derive(Debug, Clone)]
pub struct Step {
    pub at: i64,
    pub action: ActionId,
    pub actor: usize,
    pub target: Option<usize>,
    pub text: Option<String>,
}

/// `count` steps, 37 simulated seconds apart from `start`, cycling through notes about each person,
/// a brew every fifth step and a rejected (empty) note every seventh. Pure arithmetic on the index,
/// so two processes build the same script.
pub fn script(count: usize, start: i64, first_action: u64) -> Vec<Step> {
    (0..count)
        .map(|index| {
            let text = if index % 5 == 4 {
                None
            } else if index % 7 == 6 {
                Some(String::new())
            } else {
                Some(format!("note-{index}"))
            };
            Step {
                at: start + 37 * i64::try_from(index).expect("small"),
                action: ActionId::from_raw(first_action + u64::try_from(index).expect("small")),
                actor: index % PEOPLE.len(),
                target: (index % 3 != 0).then_some((index + 1) % PEOPLE.len()),
                text,
            }
        })
        .collect()
}

impl Step {
    pub fn intent(&self, people: &[EntityId]) -> ActionIntent {
        let record = match &self.text {
            Some(text) => ActionRecord::new::<Note>(encode(&Note { text: text.clone() })),
            None => ActionRecord::new::<Brew>(encode(&Brew)),
        };
        let intent = ActionIntent::new(self.action, people[self.actor], record, t(self.at));
        match self.target {
            Some(target) => intent.with_target(people[target]),
            None => intent,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// a scratch directory
// ---------------------------------------------------------------------------------------------

/// A directory of its own under the system temporary directory, emptied when created and removed
/// when dropped.
pub struct Scratch {
    path: PathBuf,
}

impl Scratch {
    pub fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "mineworld-persistence-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a scratch directory");
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
