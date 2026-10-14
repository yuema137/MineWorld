//! A crowd in one square: `n` people at one spot, with presence, conversation and relationships, driven
//! by real `talk` requests — the scripted world FA-3 measures the acquaintance cap in (D-FGA-10).

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityId, EntityKey, EntityType,
    EventEnvelope, LocalPosition, Location, Millimetres, PersonId, PlaceId, WorldTime,
};
use mineworld_conversation::{ConversationSystem, Talk, Utterance};
use mineworld_kernel::{Emission, World};
use mineworld_presence::{PresenceSystem, arrival};
use mineworld_relationships::{Acquaintances, RelationshipsSystem, knows};

use super::GENESIS;

pub struct Crowd {
    pub world: World,
    pub people: Vec<EntityId>,
    clock: i64,
    next_action: u64,
}

impl Crowd {
    /// `n` people, `p000` …, in the order of their ids, all at one spot of one square; `section` is
    /// relationships' configured fact, if any.
    pub fn new(n: usize, section: Option<Emission>) -> Self {
        let mut world = World::new();
        world.install(PresenceSystem).expect("presence");
        world.install(ConversationSystem).expect("conversation");
        world.install(RelationshipsSystem).expect("relationships");
        let square = world
            .create_entity(EntityKey::new("square").expect("key"), EntityType::Place)
            .expect("created");
        let square = PlaceId::new(square, EntityType::Place).expect("a place");
        let people: Vec<EntityId> = (0..n)
            .map(|index| {
                world
                    .create_entity(
                        EntityKey::new(format!("p{index:03}")).expect("key"),
                        EntityType::Person,
                    )
                    .expect("created")
            })
            .collect();
        let spot = Location::in_place(square).with_local(LocalPosition::on_ground(
            Millimetres::new(1_000),
            Millimetres::new(1_000),
        ));
        let mut facts: Vec<Emission> = people
            .iter()
            .map(|person| {
                let person = PersonId::new(*person, EntityType::Person).expect("a person");
                arrival(&world.read(), person, spot).expect("admitted")
            })
            .collect();
        facts.extend(section);
        world.genesis(GENESIS, facts).expect("begins");
        Self {
            world,
            people,
            clock: GENESIS.seconds(),
            next_action: 1,
        }
    }

    /// `from` says hello to `to`, a second after the last request.
    pub fn talk(&mut self, from: EntityId, to: EntityId) -> Vec<EventEnvelope> {
        self.clock += 1;
        let _due = self
            .world
            .advance_to(WorldTime::from_seconds(self.clock))
            .expect("advances");
        self.talk_now(from, to)
    }

    /// `from` says hello to `to` at the instant of the last request: two contacts at one instant.
    pub fn talk_now(&mut self, from: EntityId, to: EntityId) -> Vec<EventEnvelope> {
        let when = WorldTime::from_seconds(self.clock);
        let said = Utterance::new("Hello.").expect("an utterance");
        let record =
            ActionRecord::new::<Talk>(serde_json::to_vec(&Talk::new(said)).expect("encodes"));
        let intent = ActionIntent::new(ActionId::from_raw(self.next_action), from, record, when)
            .with_target(to);
        self.next_action += 1;
        let dispatched = self.world.dispatch(&intent, when).expect("answers");
        assert!(
            matches!(dispatched.result(), ActionResult::Accepted { .. }),
            "{:?}",
            dispatched.result()
        );
        dispatched.events().to_vec()
    }

    /// Lets `seconds` pass with no request.
    pub fn pass(&mut self, seconds: i64) {
        self.clock += seconds;
        let _due = self
            .world
            .advance_to(WorldTime::from_seconds(self.clock))
            .expect("advances");
    }

    /// Whom `holder` knows, by counterpart id.
    pub fn known(&self, holder: EntityId) -> Vec<EntityId> {
        self.world
            .read()
            .component::<Acquaintances>(holder)
            .map(|known| {
                known
                    .known()
                    .iter()
                    .map(|entry| entry.counterpart().entity_id())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whom `holder` has a `knows` edge to, sorted.
    pub fn edges_from(&self, holder: EntityId) -> Vec<EntityId> {
        let mut to: Vec<EntityId> = self
            .world
            .read()
            .relations_touching(holder)
            .filter(|edge| *edge.relation_type() == knows() && edge.from() == holder)
            .map(|edge| edge.to())
            .collect();
        to.sort();
        to
    }
}
