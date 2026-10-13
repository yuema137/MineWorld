//! IB-6 — resolution is total and independent of the order entries are written in
//! (step-18-interaction-list §12.5; `docs/DECISIONS.md` `ARC-63` item 5).
//!
//! A synthetic declaration (three roles, four classes, two places, three parameter fields, one fact)
//! and 10 000 sections drawn by SplitMix64 from a fixed seed — the tree's deterministic generator idiom
//! (`cognition/rule-controller/src/paced.rs`), no new dependency (QIB-9). Each section is either refused
//! at load, or every lookup over every class tuple × place × the fact answers; and every permutation of
//! its entries (100 seeded shuffles above five entries) resolves to the identical value, or is refused
//! with the identical kind of refusal.

use std::collections::BTreeMap;

use mineworld_authoring::{ClassDefinition, ClassName, ConfigurationRefusal, EntityClasses};
use mineworld_contracts::{
    ActionTypeId, ComponentTypeId, EntityKey, EntityType, EventTypeId, SystemId, Tag, Tags,
};
use mineworld_kernel::{System, SystemDeclaration, SystemIdentity, SystemVersion};
use mineworld_sdk::SystemPack;
use mineworld_sdk::interactions::{
    ActionDecl, Audience, ConsEntry, Effect, Entries, FactDecl, InteractionSection, ParamEntry,
    Position, Resolved, Role, RoleSubjects, RuleEntry, Section, Selector, Selectors, resolve,
};

mineworld_sdk::parameters! {
    /// The synthetic pack's three fields.
    pub struct Synthetic, partial SyntheticPartial {
        /// One.
        first: u32 = 1, 1 ..= 9;
        /// Two.
        second: u32 = 2, 1 ..= 9;
        /// Three.
        third: u32 = 3, 1 ..= 9;
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Pack;

impl SystemIdentity for Pack {
    const ID: SystemId = SystemId::from_static("synthetic");
}

impl System for Pack {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }
}

impl SystemPack for Pack {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
}

const ACT: ActionTypeId = ActionTypeId::from_static("synthetic-act");
const FACT: EventTypeId = EventTypeId::from_static("synthetic-fact");
const ROLES: [Role; 3] = [Role::Actor, Role::Target, Role::Object];

impl InteractionSection for Pack {
    type Parameters = Synthetic;
    type Knobs = ();
    const CONFIGURED: EventTypeId = EventTypeId::from_static("synthetic-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("synthetic-interactions");
    const ACTIONS: &'static [ActionDecl] = &[ActionDecl {
        action: ACT,
        roles: &ROLES,
        regional: true,
    }];
    const FACTS: &'static [FactDecl] = &[FactDecl {
        fact: FACT,
        roles: &[
            (Role::Actor, Position::Subject(0)),
            (Role::Target, Position::Subject(1)),
            (Role::Object, Position::Participant(0)),
        ],
        default_audience: Audience::Public,
        narrowest: Audience::Participants,
        biography_configurable: true,
    }];
    const PARAMETER_ROLES: &'static [Role] = &ROLES;

    fn encode(_: &Resolved<Self>) -> Vec<u8> {
        Vec::new()
    }

    fn decode(_: &[u8]) -> Result<Resolved<Self>, String> {
        Err("unused".to_owned())
    }
}

/// SplitMix64 (Steele, Lea, Flood 2014), as `kernel/tests/long_run.rs` writes it.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> usize {
        usize::try_from(self.next() % n).expect("small")
    }
}

/// The four classes: `noble` and `servant` over people, `relic` over items, and the implicit
/// `person`. (`ghost` is never defined: a draw naming it is refused as undefined.)
fn classes() -> EntityClasses {
    let definition = |class, of, tag: &str| ClassDefinition {
        class: ClassName::from_static(class),
        of,
        tag: Tag::new(tag).expect("a tag"),
    };
    EntityClasses::try_from(vec![
        definition("noble", EntityType::Person, "noble"),
        definition("servant", EntityType::Person, "servant"),
        definition("relic", EntityType::Item, "relic"),
    ])
    .expect("valid")
}

const SELECTABLE: [&str; 5] = ["noble", "servant", "relic", "person", "ghost"];

fn selectors(draw: &mut SplitMix64) -> Selectors {
    let mut selectors = Selectors::any();
    for role in ROLES {
        // About half the roles stay `*`; `ghost` is drawn rarely.
        let pick = draw.below(12);
        if pick < 4 {
            selectors = selectors.with(
                role,
                Selector::Class(ClassName::from_static(SELECTABLE[pick])),
            );
        } else if pick == 4 && draw.below(20) == 0 {
            selectors = selectors.with(role, Selector::Class(ClassName::from_static("ghost")));
        }
    }
    selectors
}

fn value(draw: &mut SplitMix64) -> Option<u32> {
    match draw.below(4) {
        0 => None,
        n => Some(u32::try_from(n).expect("small")),
    }
}

/// One drawn entry, in whichever list it belongs to.
#[derive(Clone)]
enum Drawn {
    Rule(RuleEntry),
    Param(ParamEntry<SyntheticPartial>),
    Cons(ConsEntry<()>),
}

fn entry(draw: &mut SplitMix64) -> Drawn {
    let selectors = selectors(draw);
    match draw.below(3) {
        0 => Drawn::Rule(RuleEntry {
            action: ACT,
            selectors,
            effect: if draw.below(2) == 0 {
                Effect::Permit
            } else {
                Effect::Forbid
            },
        }),
        1 => Drawn::Param(ParamEntry {
            selectors,
            fields: SyntheticPartial {
                first: value(draw),
                second: value(draw),
                third: value(draw),
            },
        }),
        _ => Drawn::Cons(ConsEntry {
            fact: FACT,
            selectors,
            audience: [None, Some(Audience::Place), Some(Audience::Participants)][draw.below(3)],
            biography: [None, Some(true), Some(false)][draw.below(3)],
            knobs: (),
        }),
    }
}

fn entries(drawn: &[Drawn]) -> Entries<Pack> {
    let mut entries = Entries::default();
    for entry in drawn {
        match entry.clone() {
            Drawn::Rule(rule) => entries.rules.push(rule),
            Drawn::Param(param) => entries.parameters.push(param),
            Drawn::Cons(cons) => entries.consequences.push(cons),
        }
    }
    entries
}

const PLACES: [&str; 2] = ["hall", "yard"];

/// A drawn section: the world's entries, and each region's.
struct Drawing {
    world: Vec<Drawn>,
    regions: BTreeMap<EntityKey, Vec<Drawn>>,
    default: Option<Effect>,
}

fn drawing(draw: &mut SplitMix64) -> Drawing {
    let world = (0..draw.below(9)).map(|_| entry(draw)).collect();
    let mut regions = BTreeMap::new();
    for place in PLACES {
        if draw.below(3) == 0 {
            let drawn = (0..draw.below(4)).map(|_| entry(draw)).collect();
            regions.insert(EntityKey::new(place).expect("a key"), drawn);
        }
    }
    let default = [None, Some(Effect::Permit), Some(Effect::Forbid)][draw.below(3)];
    Drawing {
        world,
        regions,
        default,
    }
}

fn section(drawing: &Drawing, world: &[Drawn]) -> Section<Pack> {
    let mut section = Section::empty();
    section.default = drawing.default;
    section.entries = entries(world);
    section.regions = drawing
        .regions
        .iter()
        .map(|(place, drawn)| (place.clone(), entries(drawn)))
        .collect();
    section
}

/// What a resolution is compared by: its whole value, or the kind of refusal.
fn outcome(resolved: &Result<Resolved<Pack>, ConfigurationRefusal>) -> String {
    match resolved {
        Ok(resolved) => format!("{resolved:?}"),
        Err(ConfigurationRefusal::ClassUndefined { .. }) => "ClassUndefined".to_owned(),
        Err(ConfigurationRefusal::Ambiguous { .. }) => "Ambiguous".to_owned(),
    }
}

/// Every permutation of `items`, by Heap's algorithm.
fn permutations(items: &[Drawn]) -> Vec<Vec<Drawn>> {
    fn heap(k: usize, items: &mut Vec<Drawn>, out: &mut Vec<Vec<Drawn>>) {
        if k <= 1 {
            out.push(items.clone());
            return;
        }
        for i in 0..k {
            heap(k - 1, items, out);
            let j = if k.is_multiple_of(2) { i } else { 0 };
            items.swap(j, k - 1);
        }
    }
    let mut out = Vec::new();
    heap(items.len(), &mut items.to_vec(), &mut out);
    out
}

fn shuffled(items: &[Drawn], draw: &mut SplitMix64) -> Vec<Drawn> {
    let mut items = items.to_vec();
    for i in (1..items.len()).rev() {
        let j = draw.below(u64::try_from(i + 1).expect("small"));
        items.swap(i, j);
    }
    items
}

/// Every lookup over every class tuple × place × the fact answers.
fn every_lookup_answers(resolved: &Resolved<Pack>, classes: &EntityClasses) -> usize {
    let people = [
        Tags::new([Tag::new("noble").expect("t")]),
        Tags::new([Tag::new("servant").expect("t")]),
        Tags::default(),
    ];
    let items = [Tags::new([Tag::new("relic").expect("t")]), Tags::default()];
    let mut answers = 0;
    let places: Vec<Option<EntityKey>> = std::iter::once(None)
        .chain(
            PLACES
                .iter()
                .map(|place| Some(EntityKey::new(*place).expect("a key"))),
        )
        .collect();
    for place in &places {
        let resolution = resolved.at(place.as_ref());
        for actor in &people {
            for target in &people {
                for object in &items {
                    let subjects = RoleSubjects::none()
                        .with(Role::Actor, EntityType::Person, actor)
                        .with(Role::Target, EntityType::Person, target)
                        .with(Role::Object, EntityType::Item, object);
                    let _ = resolution.permits(classes, &ACT, &subjects);
                    let _ = resolution.parameters(classes, &subjects);
                    let _ = resolution.consequence(classes, &FACT, &subjects);
                    answers += 3;
                }
            }
        }
    }
    answers
}

#[test]
fn every_section_is_refused_or_answers_everything_whatever_its_entry_order() {
    let classes = classes();
    let mut draw = SplitMix64(0x1b_2026_1008);
    let mut refused: BTreeMap<String, usize> = BTreeMap::new();
    let (mut answered, mut orders) = (0_usize, 0_usize);
    for case in 0..10_000 {
        let drawing = drawing(&mut draw);
        let first = resolve(&section(&drawing, &drawing.world), &classes);
        let expected = outcome(&first);
        match &first {
            Ok(resolved) => answered += every_lookup_answers(resolved, &classes),
            Err(_) => *refused.entry(expected.clone()).or_default() += 1,
        }
        let orderings = if drawing.world.len() <= 5 {
            permutations(&drawing.world)
        } else {
            (0..100)
                .map(|_| shuffled(&drawing.world, &mut draw))
                .collect()
        };
        for order in orderings {
            orders += 1;
            assert_eq!(
                outcome(&resolve(&section(&drawing, &order), &classes)),
                expected,
                "case {case}: the resolution depends on the order entries are written in"
            );
        }
    }
    println!(
        "IB-6: 10000 sections; refused {refused:?}; {answered} lookups answered; {orders} orders"
    );
    assert!(refused.contains_key("Ambiguous") && refused.contains_key("ClassUndefined"));
    assert!(answered > 0);
}
