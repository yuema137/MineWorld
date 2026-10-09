//! Each precedence rule and each refusal of resolution, on a probe pack's sections built in code
//! (decoding with line and column is proven through the loader, in worldpack's in-crate tests).

use mineworld_authoring::{ClassDefinition, ClassName, ConfigurationRefusal, EntityClasses};
use mineworld_contracts::{ActionTypeId, EntityKey, EntityType, EventTypeId, SystemId, Tag, Tags};
use mineworld_kernel::{SystemDeclaration, SystemIdentity, SystemVersion};

use super::*;

crate::parameters! {
    /// The probe's parameters.
    pub struct ProbeParameters, partial ProbePartial {
        /// A step.
        step: u32 = 5, 1 ..= 100;
        /// A reach.
        reach: u32 = 10, 1 ..= 1_000;
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Probe;

impl SystemIdentity for Probe {
    const ID: SystemId = SystemId::from_static("probe");
}

impl mineworld_kernel::System for Probe {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }
}

impl crate::SystemPack for Probe {
    const PACKAGE: crate::Package = crate::package!();
}

const ACT: ActionTypeId = ActionTypeId::from_static("probe-act");
const DONE: EventTypeId = EventTypeId::from_static("probe-done");

impl InteractionSection for Probe {
    type Parameters = ProbeParameters;
    type Knobs = ();
    const CONFIGURED: EventTypeId = EventTypeId::from_static("probe-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("probe-interactions");
    const ACTIONS: &'static [ActionDecl] = &[ActionDecl {
        action: ACT,
        roles: &[Role::Actor, Role::Target],
        regional: true,
    }];
    const FACTS: &'static [FactDecl] = &[FactDecl {
        fact: DONE,
        roles: &[(Role::Actor, Position::Subject(0))],
        default_audience: Audience::Place,
        narrowest: Audience::Participants,
        biography_configurable: true,
    }];
    const PARAMETER_ROLES: &'static [Role] = &[Role::Actor, Role::Target];

    fn reference_lists() -> Vec<(&'static str, Section<Self>)> {
        let mut default = Section::empty();
        default
            .entries
            .parameters
            .push(param(Selectors::any(), Some(7), None));
        vec![("default", default)]
    }

    fn encode(_: &Resolved<Self>) -> Vec<u8> {
        Vec::new()
    }

    fn decode(_: &[u8]) -> Result<Resolved<Self>, String> {
        Err("not in a unit test".to_owned())
    }
}

fn class(name: &'static str) -> Selector {
    Selector::Class(ClassName::from_static(name))
}

fn param(selectors: Selectors, step: Option<u32>, reach: Option<u32>) -> ParamEntry<ProbePartial> {
    ParamEntry {
        selectors,
        fields: ProbePartial { step, reach },
    }
}

fn rule(selectors: Selectors, effect: Effect) -> RuleEntry {
    RuleEntry {
        action: ACT,
        selectors,
        effect,
    }
}

fn classes() -> EntityClasses {
    let definition = |class, of, tag: &str| ClassDefinition {
        class: ClassName::from_static(class),
        of,
        tag: Tag::new(tag).expect("a tag"),
    };
    EntityClasses::try_from(vec![
        definition("noble", EntityType::Person, "noble"),
        definition("servant", EntityType::Person, "servant"),
        definition("hall", EntityType::Place, "noble"),
    ])
    .expect("valid")
}

fn tags(values: &[&str]) -> Tags {
    Tags::new(values.iter().map(|value| Tag::new(*value).expect("a tag")))
}

fn actor(selector: Selector) -> Selectors {
    Selectors::any().with(Role::Actor, selector)
}

/// Levels replace by key, field by field: the default list's step is replaced by the world's, and the
/// reach the world does not name keeps the compiled default. Absent `parameters`, the base is the
/// default list's (M-IB2's target).
#[test]
fn a_higher_level_replaces_a_lower_levels_entry_field_by_field() {
    let mut section = Section::<Probe>::empty();
    let resolved = resolve(&section, &classes()).expect("resolves");
    assert_eq!(
        resolved.base.parameters,
        ProbeParameters { step: 7, reach: 10 }
    );

    section
        .entries
        .parameters
        .push(param(Selectors::any(), Some(9), None));
    let resolved = resolve(&section, &classes()).expect("resolves");
    assert_eq!(
        resolved.base.parameters,
        ProbeParameters { step: 9, reach: 10 }
    );
}

/// Specificity: the entry naming most roles wins, field by field; an implicit class counts as named.
#[test]
fn the_most_specific_matching_entry_wins_field_by_field() {
    let mut section = Section::<Probe>::empty();
    section.entries.parameters.extend([
        param(actor(class("person")), Some(20), Some(200)),
        param(
            actor(class("noble")).with(Role::Target, Selector::Any),
            Some(20),
            None,
        ),
        param(
            actor(class("servant")).with(Role::Target, class("noble")),
            Some(30),
            None,
        ),
    ]);
    let classes = classes();
    let resolved = resolve(&section, &classes).expect("resolves");
    let (noble, servant, plain) = (tags(&["noble"]), tags(&["servant"]), tags(&[]));
    let lookup = |actor: &Tags, target: &Tags| {
        resolved.base.parameters(
            &classes,
            &RoleSubjects::none()
                .with(Role::Actor, EntityType::Person, actor)
                .with(Role::Target, EntityType::Person, target),
        )
    };
    assert_eq!(
        lookup(&plain, &plain),
        ProbeParameters {
            step: 20,
            reach: 200
        }
    );
    assert_eq!(
        lookup(&servant, &plain),
        ProbeParameters {
            step: 20,
            reach: 200
        }
    );
    assert_eq!(
        lookup(&servant, &noble),
        ProbeParameters {
            step: 30,
            reach: 200
        }
    );
}

/// Rules: forbid overrides permit at equal specificity (M-IB5a's target); a more specific permit
/// beats a less specific forbid; with no matching rule the default applies, and `default: forbid` denies.
#[test]
fn forbid_wins_ties_specificity_wins_otherwise_and_the_default_applies_last() {
    let classes = classes();
    let (noble, servant) = (tags(&["noble"]), tags(&["servant"]));
    let subjects = |actor: &'static Tags| -> RoleSubjects<'static> {
        RoleSubjects::none().with(Role::Actor, EntityType::Person, actor)
    };
    let noble: &'static Tags = Box::leak(Box::new(noble));
    let servant: &'static Tags = Box::leak(Box::new(servant));

    let mut section = Section::<Probe>::empty();
    section.entries.rules.extend([
        rule(actor(class("noble")), Effect::Permit),
        rule(actor(class("noble")), Effect::Forbid),
    ]);
    let resolved = resolve(&section, &classes).expect("resolves");
    assert!(
        !resolved.base.permits(&classes, &ACT, &subjects(noble)),
        "forbid wins"
    );
    assert!(
        resolved.base.permits(&classes, &ACT, &subjects(servant)),
        "default permit"
    );

    let mut section = Section::<Probe>::empty();
    section.default = Some(Effect::Forbid);
    section.entries.rules.extend([
        rule(actor(class("person")), Effect::Forbid),
        rule(
            actor(class("noble")).with(Role::Target, Selector::Any),
            Effect::Forbid,
        ),
        rule(
            actor(class("noble")).with(Role::Target, class("servant")),
            Effect::Permit,
        ),
    ]);
    let resolved = resolve(&section, &classes).expect("resolves");
    let pair = RoleSubjects::none()
        .with(Role::Actor, EntityType::Person, noble)
        .with(Role::Target, EntityType::Person, servant);
    assert!(
        resolved.base.permits(&classes, &ACT, &pair),
        "more specific permit"
    );
    assert!(!resolved.base.permits(&classes, &ACT, &subjects(noble)));

    let mut section = Section::<Probe>::empty();
    section.default = Some(Effect::Forbid);
    let resolved = resolve(&section, &classes).expect("resolves");
    assert!(!resolved.base.permits(&classes, &ACT, &RoleSubjects::none()));
}

/// Equal specificity, overlapping, disagreeing: refused, naming both entries. Overlap includes an
/// implicit class against a class of its type; two classes of one type do not overlap.
#[test]
fn ambiguous_entries_are_refused_and_disjoint_classes_are_not_ambiguous() {
    let classes = classes();
    let mut section = Section::<Probe>::empty();
    section.entries.parameters.extend([
        param(actor(class("noble")), Some(20), None),
        param(actor(class("servant")), Some(30), None),
    ]);
    resolve(&section, &classes).expect("two classes of one type are disjoint");

    section
        .entries
        .parameters
        .push(param(actor(class("person")), Some(40), None));
    match resolve(&section, &classes) {
        Err(ConfigurationRefusal::Ambiguous {
            first,
            second,
            field,
        }) => {
            assert_eq!(field, "step");
            assert_eq!(first.list, "parameters");
            assert_eq!(second.list, "parameters");
            assert_ne!(first.index, second.index);
        }
        other => panic!("expected Ambiguous, got {other:?}"),
    }

    let mut agree = Section::<Probe>::empty();
    agree.entries.parameters.extend([
        param(actor(class("noble")), Some(20), None),
        param(actor(class("person")), None, Some(50)),
    ]);
    resolve(&agree, &classes).expect("overlapping entries that set different fields agree");
}

/// A selector naming a class neither defined nor implicit is refused with its list and index.
#[test]
fn an_undefined_class_is_refused_with_its_entry() {
    let mut section = Section::<Probe>::empty();
    section.regions.insert(
        EntityKey::new("cafe").expect("a key"),
        Entries {
            rules: vec![rule(actor(class("knight")), Effect::Forbid)],
            ..Entries::default()
        },
    );
    match resolve(&section, &classes()) {
        Err(ConfigurationRefusal::ClassUndefined { class, entry }) => {
            assert_eq!(class.as_str(), "knight");
            assert_eq!(entry.to_string(), "regions.cafe.rules[0]");
        }
        other => panic!("expected ClassUndefined, got {other:?}"),
    }
}

/// A region's entries apply at its place only, over the world's (M-IB4a's target).
#[test]
fn a_region_overrides_only_at_its_place() {
    let mut section = Section::<Probe>::empty();
    let cafe = EntityKey::new("cafe").expect("a key");
    section.regions.insert(
        cafe.clone(),
        Entries {
            parameters: vec![param(Selectors::any(), Some(60), None)],
            ..Entries::default()
        },
    );
    let resolved = resolve(&section, &classes()).expect("resolves");
    assert_eq!(resolved.at(Some(&cafe)).parameters.step, 60);
    assert_eq!(resolved.at(None).parameters.step, 7);
    assert_eq!(
        resolved
            .at(Some(&EntityKey::new("park").expect("a key")))
            .parameters
            .step,
        7
    );
}

/// The resolved fact copies the classes referenced and those that could shadow them (M-IB8's
/// target), and nothing else.
#[test]
fn the_resolved_section_copies_what_its_selectors_can_be_decided_by() {
    let mut section = Section::<Probe>::empty();
    section
        .entries
        .parameters
        .push(param(actor(class("servant")), Some(3), None));
    let resolved = resolve(&section, &classes()).expect("resolves");
    let kept: Vec<&str> = resolved
        .classes
        .definitions()
        .iter()
        .map(|definition| definition.class.as_str())
        .collect();
    assert_eq!(kept, ["noble", "servant"]);
}
