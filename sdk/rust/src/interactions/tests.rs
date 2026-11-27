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
    // `reach` is about the actor alone: no target, no place, no region.
    const FIELD_ROLES: &'static [(&'static str, &'static [Role])] = &[("reach", &[Role::Actor])];

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

/// `rule_effect` tells a matching rule from no rule, with the same precedence as `permits`, which is
/// `rule_effect` with the default filling the gap (SD-Na-3, the lookup `ruling` reads).
#[test]
fn rule_effect_tells_a_matching_rule_from_the_default() {
    let classes = classes();
    let noble: &'static Tags = Box::leak(Box::new(tags(&["noble"])));
    let servant: &'static Tags = Box::leak(Box::new(tags(&["servant"])));
    let subjects = |actor: &'static Tags| -> RoleSubjects<'static> {
        RoleSubjects::none().with(Role::Actor, EntityType::Person, actor)
    };

    let unsaid = resolve(&Section::<Probe>::empty(), &classes).expect("resolves");
    assert_eq!(
        unsaid.base.rule_effect(&classes, &ACT, &subjects(noble)),
        None
    );
    assert_eq!(unsaid.base.default, Effect::Permit, "permit when unsaid");

    let mut section = Section::<Probe>::empty();
    section.entries.rules.extend([
        rule(actor(class("noble")), Effect::Permit),
        rule(actor(class("noble")), Effect::Forbid),
    ]);
    let resolved = resolve(&section, &classes).expect("resolves");
    assert_eq!(
        resolved.base.rule_effect(&classes, &ACT, &subjects(noble)),
        Some(Effect::Forbid),
        "forbid wins among equals"
    );
    assert_eq!(
        resolved
            .base
            .rule_effect(&classes, &ACT, &subjects(servant)),
        None,
        "no rule matches a servant"
    );

    let mut section = Section::<Probe>::empty();
    section.default = Some(Effect::Forbid);
    section
        .entries
        .rules
        .push(rule(actor(class("noble")), Effect::Permit));
    let resolved = resolve(&section, &classes).expect("resolves");
    assert_eq!(
        resolved.base.rule_effect(&classes, &ACT, &subjects(noble)),
        Some(Effect::Permit)
    );
    assert!(resolved.base.permits(&classes, &ACT, &subjects(noble)));
    assert_eq!(
        resolved
            .base
            .rule_effect(&classes, &ACT, &subjects(servant)),
        None
    );
    assert_eq!(resolved.base.default, Effect::Forbid);
    assert!(!resolved.base.permits(&classes, &ACT, &subjects(servant)));
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

/// A section's schema says what its decoder accepts (`ARC-90`): the example it carries decodes, the
/// parameter fields carry `parameters!`'s defaults and bounds and the doc comment, a region's rules
/// list only regional actions, and a key the decoder refuses is not a property.
#[test]
fn a_sections_schema_states_what_its_decoder_accepts() {
    use schemars::{JsonSchema, SchemaGenerator};

    let schema = SchemaGenerator::default().into_root_schema_for::<Section<Probe>>();
    let example = schema
        .get("examples")
        .and_then(|examples| examples.get(0))
        .cloned()
        .expect("an example");
    let decoded = Section::<Probe>::deserialize(example.clone()).expect("the example decodes");
    assert_eq!(decoded.entries.rules.len(), 1, "{example}");
    assert_eq!(decoded.entries.parameters.len(), 1, "{example}");

    let step = schema
        .pointer("/properties/parameters/items/properties/step")
        .expect("the step parameter");
    assert_eq!(step["default"], 5);
    assert_eq!(step["minimum"], 1);
    assert_eq!(step["maximum"], 100);
    assert_eq!(step["description"], "A step.");
    let actions = schema
        .pointer("/properties/regions/additionalProperties/properties/rules/items/properties/action/enum")
        .expect("a region's actions");
    assert_eq!(actions, &serde_json::json!(["probe-act"]));
    let keys: Vec<&String> = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .expect("properties")
        .keys()
        .collect();
    assert_eq!(
        keys,
        [
            "consequences",
            "default",
            "extends",
            "parameters",
            "regions",
            "rules"
        ]
    );
    let partial = ProbePartial::json_schema(&mut SchemaGenerator::default());
    assert_eq!(
        partial.get("additionalProperties"),
        Some(&serde_json::json!(false))
    );
}

/// A range bound words its refusal exactly as every refusal was worded before named choices (S1,
/// R-FGA-1): `low … high`, and admits both ends.
#[test]
fn a_range_bound_admits_its_ends_and_words_itself_as_before() {
    let bound = 1_u32..=100;
    assert!(ParameterBound::admits(&bound, &1) && ParameterBound::admits(&bound, &100));
    assert!(!ParameterBound::admits(&bound, &0) && !ParameterBound::admits(&bound, &101));
    assert_eq!(ParameterBound::<u32>::describe(&bound), "1 … 100");
    assert_eq!(
        ParameterBound::<i32>::describe(&(-1_000..=1_000)),
        "-1000 … 1000"
    );
    assert_eq!(
        ParameterBound::<bool>::describe(&(false..=true)),
        "false … true"
    );
}

/// Per-field roles (S2): a field its owner lists may be scoped only by its roles and set in a region
/// only if `place` is one; an unlisted field keeps every parameter role; an entry setting nothing
/// restricted is never refused.
#[test]
fn a_restricted_field_is_refused_under_another_role_and_in_a_region() {
    let set = |step, reach| ProbePartial { step, reach }.set_names();
    assert_eq!(set(Some(1), Some(2)), ["step", "reach"]);
    assert_eq!(set(None, None), Vec::<&str>::new());

    let reach = set(None, Some(2));
    assert_eq!(
        section::field_roles::<Probe>(&reach, &[Role::Actor], false),
        Ok(())
    );
    assert_eq!(
        section::field_roles::<Probe>(&reach, &[Role::Actor, Role::Target], false),
        Err(
            "'reach' of 'probe' may not be scoped by the role 'target' (its roles: actor)"
                .to_owned()
        )
    );
    assert_eq!(
        section::field_roles::<Probe>(&reach, &[], true),
        Err("a region may not set 'reach': 'probe' does not let a place scope it (its roles: actor)".to_owned())
    );
    let step = set(Some(1), None);
    assert_eq!(
        section::field_roles::<Probe>(&step, &[Role::Target], true),
        Ok(())
    );
}

// ---- An owner's own versions and refusal (`ARC-370`) ---------------------------------------------

/// A second probe that raises its section's schema versions and refuses `reach` anywhere while its
/// own base sets `step` to 100 — a combination neither field's bound can express.
#[derive(Debug, Clone, Default, PartialEq)]
struct Strict;

impl SystemIdentity for Strict {
    const ID: SystemId = SystemId::from_static("strict");
}

impl mineworld_kernel::System for Strict {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        declare::<Self>(SystemDeclaration::of::<Self>())
    }
}

impl crate::SystemPack for Strict {
    const PACKAGE: crate::Package = crate::package!();
}

const STRICT_CODE: RejectionCode = RejectionCode::from_static("strict-reach-with-step-100");

std::thread_local! {
    /// How many times `Strict::validate_authored` ran on this thread.
    static VALIDATIONS: core::cell::Cell<usize> = const { core::cell::Cell::new(0) };
}

impl InteractionSection for Strict {
    type Parameters = ProbeParameters;
    type Knobs = ();
    const CONFIGURED: EventTypeId = EventTypeId::from_static("strict-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("strict-interactions");
    const CONFIGURED_SCHEMA_VERSION: mineworld_contracts::EventSchemaVersion =
        mineworld_contracts::EventSchemaVersion::new(2);
    const COMPONENT_SCHEMA_VERSION: mineworld_contracts::ComponentSchemaVersion =
        mineworld_contracts::ComponentSchemaVersion::new(2);
    const PARAMETER_ROLES: &'static [Role] = &[Role::Actor];

    /// `default` sets `reach`; `generous` extends `wide`, which sets it too.
    fn reference_lists() -> Vec<(&'static str, Section<Self>)> {
        let mut default = Section::empty();
        default
            .entries
            .parameters
            .push(param(Selectors::any(), None, Some(11)));
        let mut wide = Section::empty();
        wide.entries
            .parameters
            .push(param(Selectors::any(), None, Some(12)));
        let mut generous = Section::empty();
        generous.extends = Some("wide");
        generous
            .entries
            .parameters
            .push(param(Selectors::any(), Some(3), None));
        vec![("default", default), ("wide", wide), ("generous", generous)]
    }

    fn validate_authored(
        section: &Section<Self>,
        resolved: &Resolved<Self>,
    ) -> Result<(), ConfigurationRefusal> {
        VALIDATIONS.with(|count| count.set(count.get() + 1));
        if resolved.base.parameters.step != 100 {
            return Ok(());
        }
        match section.entries_setting("reach").into_iter().next() {
            None => Ok(()),
            Some(entry) => Err(ConfigurationRefusal::Owner {
                entry: Some(entry),
                code: STRICT_CODE,
                detail: "'reach' may not be set beside step 100".to_owned(),
            }),
        }
    }

    fn encode(_: &Resolved<Self>) -> Vec<u8> {
        Vec::new()
    }

    fn decode(_: &[u8]) -> Result<Resolved<Self>, String> {
        Err("not in a unit test".to_owned())
    }
}

fn validations() -> usize {
    VALIDATIONS.with(core::cell::Cell::get)
}

/// An owner that declares version 2 states its configured fact and declares its component at 2; an
/// owner that declares nothing stays at 1 (E5: unrelated packs keep schema 1).
#[test]
fn an_owners_declared_versions_reach_its_fact_and_component_and_no_other_pack() {
    let configured =
        |emission: mineworld_kernel::Emission| emission.record().schema_version().get();
    assert_eq!(
        configured(
            mineworld_kernel::Emission::new::<SectionConfigured<Strict>>(
                Vec::new(),
                mineworld_contracts::Visibility::SystemInternal
            )
        ),
        2
    );
    assert_eq!(
        configured(mineworld_kernel::Emission::new::<SectionConfigured<Probe>>(
            Vec::new(),
            mineworld_contracts::Visibility::SystemInternal
        )),
        1
    );
    let owned = |declaration: SystemDeclaration| {
        declaration
            .owns()
            .iter()
            .map(|owned| owned.schema_version().value())
            .collect::<Vec<_>>()
    };
    assert_eq!(owned(mineworld_kernel::System::declaration(&Strict)), [2]);
    assert_eq!(
        owned(declare::<Probe>(SystemDeclaration::of::<Probe>())),
        [1]
    );
}

/// `entries_setting` names every entry setting a field, wherever it applies from — the default list,
/// the `extends` chain, the section's own entries, a region — even when a later entry shadows it.
#[test]
fn entries_setting_names_each_source_as_resolution_names_entries() {
    let mut section = Section::<Strict>::empty();
    section.extends = Some("generous");
    section.entries.parameters.extend([
        param(Selectors::any(), Some(4), None),
        param(actor(class("noble")), None, Some(13)),
    ]);
    section.regions.insert(
        EntityKey::new("cafe").expect("a key"),
        Entries {
            parameters: vec![param(Selectors::any(), None, Some(14))],
            ..Entries::default()
        },
    );
    let named: Vec<String> = section
        .entries_setting("reach")
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        named,
        [
            "reference list 'default' parameters[0]",
            "reference list 'wide' parameters[0]",
            "parameters[1]",
            "regions.cafe.parameters[0]",
        ]
    );
    assert_eq!(
        section
            .entries_setting("step")
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["reference list 'generous' parameters[0]", "parameters[0]"]
    );
}

/// The owner's refusal comes after the class and ambiguity checks, runs exactly once per resolution,
/// and names the entry the owner blames — here the default list's, which the author never wrote but
/// which applies all the same. A pack with no hook of its own refuses nothing more.
#[test]
fn an_owners_refusal_follows_the_shared_checks_and_runs_once() {
    let classes = classes();
    let before = validations();
    let mut undefined = Section::<Strict>::empty();
    undefined
        .entries
        .parameters
        .push(param(actor(class("knight")), Some(100), None));
    assert!(matches!(
        resolve(&undefined, &classes),
        Err(ConfigurationRefusal::ClassUndefined { .. })
    ));
    let mut ambiguous = Section::<Strict>::empty();
    ambiguous.entries.parameters.extend([
        param(actor(class("noble")), Some(100), None),
        param(actor(class("person")), Some(99), None),
    ]);
    assert!(matches!(
        resolve(&ambiguous, &classes),
        Err(ConfigurationRefusal::Ambiguous { .. })
    ));
    assert_eq!(validations(), before, "the shared refusals win first");

    let mut contradictory = Section::<Strict>::empty();
    contradictory
        .entries
        .parameters
        .push(param(Selectors::any(), Some(100), None));
    match resolve(&contradictory, &classes) {
        Err(ConfigurationRefusal::Owner {
            entry,
            code,
            detail,
        }) => {
            assert_eq!(
                entry.map(|entry| entry.to_string()).as_deref(),
                Some("reference list 'default' parameters[0]")
            );
            assert_eq!(code, STRICT_CODE);
            assert!(detail.contains("reach"), "{detail}");
        }
        other => panic!("expected the owner's refusal, got {other:?}"),
    }
    assert_eq!(validations(), before + 1, "exactly once per resolution");

    let mut accepted = Section::<Strict>::empty();
    accepted
        .entries
        .parameters
        .push(param(Selectors::any(), Some(99), None));
    resolve(&accepted, &classes).expect("step 99 beside reach is fine");
    assert_eq!(validations(), before + 2);

    let mut probe = Section::<Probe>::empty();
    probe
        .entries
        .parameters
        .push(param(Selectors::any(), Some(100), Some(1)));
    resolve(&probe, &classes).expect("the default hook refuses nothing");
}
