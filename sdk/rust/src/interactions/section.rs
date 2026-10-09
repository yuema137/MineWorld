//! A section as authored, and its decoding: the shape every pack shares (`ARC-63` items 1 and 3).
//!
//! Decoding *is* the first half of validation: every refusal that a pack's own declarations decide —
//! an undeclared action, fact or role, a parameter outside its bound, a widened audience, biography on
//! a fact whose owner does not allow it, a region rule for an action that is not regional, a bad
//! `extends` — is raised while the decoder stands at the offending value, so it keeps its line and
//! column (`DEP-10`). What needs the world's classes is checked after (`super::resolve::check`).

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use mineworld_contracts::{ActionTypeId, EntityKey, EventTypeId};
use serde::de::{DeserializeOwned, DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use super::InteractionSection;
use super::decl::{Audience, Effect, Role};
use super::selector::{Selector, Selectors};

/// The longest `extends` chain a section may name (`ARC-63` item 4).
pub const EXTENDS_MAX: usize = 4;

/// A set of optional, typed fields: a parameter block's partial twin, or a pack's consequence knobs.
/// Made by [`parameters!`](crate::parameters) for parameters; `()` has none.
pub trait Fields:
    Default + Clone + PartialEq + Debug + Serialize + DeserializeOwned + Send + Sync + 'static
{
    /// The fields' names, as a section writes them.
    const NAMES: &'static [&'static str];

    /// Decodes the value of the field `name` from `map`, checking its bound; `Ok(false)` when there is
    /// no such field.
    ///
    /// # Errors
    ///
    /// The decoder's error, or a value outside the field's bound.
    fn decode_field<'de, A: MapAccess<'de>>(
        &mut self,
        name: &str,
        map: &mut A,
    ) -> Result<bool, A::Error>;

    /// Sets every field `other` sets.
    fn overlay(&mut self, other: &Self);

    /// The first field both set, to different values.
    fn conflict(&self, other: &Self) -> Option<&'static str>;
}

impl Fields for () {
    const NAMES: &'static [&'static str] = &[];

    fn decode_field<'de, A: MapAccess<'de>>(
        &mut self,
        _: &str,
        _: &mut A,
    ) -> Result<bool, A::Error> {
        Ok(false)
    }

    fn overlay(&mut self, _: &Self) {}

    fn conflict(&self, _: &Self) -> Option<&'static str> {
        None
    }
}

/// A pack's typed parameter block, every field with a bound and a default (`ARC-63` item 2). Made by
/// [`parameters!`](crate::parameters).
pub trait Parameters:
    Default + Clone + PartialEq + Debug + Serialize + DeserializeOwned + Send + Sync + 'static
{
    /// The all-optional twin a scoped entry is written in.
    type Partial: Fields;

    /// Sets every field `partial` sets.
    fn apply(&mut self, partial: &Self::Partial);
}

/// Declares a pack's parameter block and its partial twin: each field's type, default and inclusive
/// bound. Used by a pack beside its `InteractionSection` impl; the invoking crate depends on `serde`.
///
/// ```text
/// mineworld_sdk::parameters! {
///     /// What a world may choose about this pack.
///     pub struct Tuning, partial TuningPartial {
///         /// Seconds of silence that end a conversation.
///         gap: u32 = 300, 1 ..= 86_400;
///     }
/// }
/// ```
#[macro_export]
macro_rules! parameters {
    (
        $(#[$meta:meta])*
        $vis:vis struct $Name:ident, partial $Partial:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident : $ty:ty = $default:literal, $low:literal ..= $high:literal;
            )+
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
        $vis struct $Name {
            $(
                $(#[$field_meta])*
                pub $field: $ty,
            )+
        }

        impl ::core::default::Default for $Name {
            fn default() -> Self {
                Self { $( $field: $default, )+ }
            }
        }

        #[doc = concat!("The all-optional twin of [`", stringify!($Name), "`]: what a scoped entry sets.")]
        #[derive(Debug, Clone, Default, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
        $vis struct $Partial {
            $(
                #[allow(missing_docs)]
                #[serde(default, skip_serializing_if = "Option::is_none")]
                pub $field: ::core::option::Option<$ty>,
            )+
        }

        impl $crate::interactions::Fields for $Partial {
            const NAMES: &'static [&'static str] = &[ $( stringify!($field), )+ ];

            fn decode_field<'de, A: $crate::__private::MapAccess<'de>>(
                &mut self,
                name: &str,
                map: &mut A,
            ) -> ::core::result::Result<bool, A::Error> {
                $(
                    if name == stringify!($field) {
                        let value: $ty = map.next_value()?;
                        if !($low..=$high).contains(&value) {
                            return ::core::result::Result::Err(
                                <A::Error as $crate::__private::DeError>::custom(format!(
                                    "'{}' is {} … {}, not {}",
                                    stringify!($field), $low, $high, value
                                )),
                            );
                        }
                        self.$field = ::core::option::Option::Some(value);
                        return ::core::result::Result::Ok(true);
                    }
                )+
                ::core::result::Result::Ok(false)
            }

            fn overlay(&mut self, other: &Self) {
                $(
                    if other.$field.is_some() {
                        self.$field = other.$field.clone();
                    }
                )+
            }

            fn conflict(&self, other: &Self) -> ::core::option::Option<&'static str> {
                $(
                    if let (::core::option::Option::Some(a), ::core::option::Option::Some(b)) =
                        (&self.$field, &other.$field)
                    {
                        if a != b {
                            return ::core::option::Option::Some(stringify!($field));
                        }
                    }
                )+
                ::core::option::Option::None
            }
        }

        impl $crate::interactions::Parameters for $Name {
            type Partial = $Partial;

            fn apply(&mut self, partial: &$Partial) {
                $(
                    if let ::core::option::Option::Some(value) = &partial.$field {
                        self.$field = value.clone();
                    }
                )+
            }
        }
    };
}

/// A parameter block's partial twin, for a section of `S`.
pub type Partial<S> = <<S as InteractionSection>::Parameters as Parameters>::Partial;

/// A rule: an action, its selectors, and what it says.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RuleEntry {
    /// The action.
    pub action: ActionTypeId,
    /// One selector per role.
    pub selectors: Selectors,
    /// Permit or forbid.
    pub effect: Effect,
}

/// A parameter entry: its selectors, and the fields it sets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "P: Fields")]
pub struct ParamEntry<P> {
    /// One selector per role; all `*` is the section's base.
    pub selectors: Selectors,
    /// The fields it sets.
    pub fields: P,
}

/// A consequence entry: a fact, its selectors, and what it sets (`ARC-65`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "K: Fields")]
pub struct ConsEntry<K> {
    /// The fact type.
    pub fact: EventTypeId,
    /// One selector per role.
    pub selectors: Selectors,
    /// The audience, narrowed.
    pub audience: Option<Audience>,
    /// Whether it is biographical.
    pub biography: Option<bool>,
    /// The pack's knobs.
    pub knobs: K,
}

/// One level's entries: a section's own, a region's, or a reference list's.
#[derive(Debug, Clone, PartialEq)]
pub struct Entries<S: InteractionSection> {
    /// Its rules, in authoring order.
    pub rules: Vec<RuleEntry>,
    /// Its parameter entries, in authoring order.
    pub parameters: Vec<ParamEntry<Partial<S>>>,
    /// Its consequence entries, in authoring order.
    pub consequences: Vec<ConsEntry<S::Knobs>>,
}

impl<S: InteractionSection> Default for Entries<S> {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            parameters: Vec::new(),
            consequences: Vec::new(),
        }
    }
}

/// A section, decoded: what `configure/<pack>.yaml` says, or one of a pack's reference lists.
#[derive(Debug, Clone, PartialEq)]
pub struct Section<S: InteractionSection> {
    /// The reference list it extends.
    pub extends: Option<&'static str>,
    /// What an action no rule matches gets; `permit` when unsaid.
    pub default: Option<Effect>,
    /// Its own entries.
    pub entries: Entries<S>,
    /// Each region's entries, by place key.
    pub regions: BTreeMap<EntityKey, Entries<S>>,
}

impl<S: InteractionSection> Default for Section<S> {
    fn default() -> Self {
        Self {
            extends: None,
            default: None,
            entries: Entries::default(),
            regions: BTreeMap::new(),
        }
    }
}

impl<S: InteractionSection> Section<S> {
    /// The empty section: a pack's `default` reference list when today's behaviour is its parameters'
    /// defaults.
    pub fn empty() -> Self {
        Self::default()
    }

    /// The reference list a world's `extends` names, its chain followed: the lists from the root-most
    /// first, `default` excluded. Refused when the name is no list of the pack, the chain cycles, or it
    /// is longer than [`EXTENDS_MAX`].
    ///
    /// # Errors
    ///
    /// The refusal, worded for the author.
    pub fn chain(name: &str) -> Result<Vec<(&'static str, Self)>, String> {
        let lists = S::reference_lists();
        let mut chain: Vec<(&'static str, Self)> = Vec::new();
        let mut next = Some(name.to_owned());
        while let Some(name) = next {
            if chain.iter().any(|(seen, _)| *seen == name) {
                return Err(format!(
                    "extends '{name}' of '{}' cycles: a reference list extends itself",
                    S::ID.as_str()
                ));
            }
            if chain.len() == EXTENDS_MAX {
                return Err(format!(
                    "the extends chain of '{}' is longer than {EXTENDS_MAX} lists",
                    S::ID.as_str()
                ));
            }
            let Some((found, list)) = lists.iter().find(|(list, _)| *list == name) else {
                let known: Vec<&str> = lists.iter().map(|(list, _)| *list).collect();
                return Err(format!(
                    "extends '{name}' names no reference list of '{}' (it has: {})",
                    S::ID.as_str(),
                    known.join(", ")
                ));
            };
            next = list.extends.map(str::to_owned);
            chain.push((found, list.clone()));
        }
        chain.reverse();
        chain.retain(|(name, _)| *name != "default");
        Ok(chain)
    }
}

// ---- decoding -----------------------------------------------------------------------------------

impl<'de, S: InteractionSection> Deserialize<'de> for Section<S> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(SectionVisitor::<S>(PhantomData))
    }
}

struct SectionVisitor<S>(PhantomData<fn() -> S>);

impl<'de, S: InteractionSection> Visitor<'de> for SectionVisitor<S> {
    type Value = Section<S>;

    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "the section of '{}'", S::ID.as_str())
    }

    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(Section::default())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut section = Section::<S>::default();
        while let Some(SectionKey(key)) = map.next_key::<SectionKey>()? {
            match key {
                "extends" => {
                    section.extends = Some(map.next_value_seed(Extends::<S>(PhantomData))?);
                }
                "default" => section.default = Some(map.next_value()?),
                "rules" => section.entries.rules = map.next_value_seed(Rules::<S>::new(false))?,
                "parameters" => {
                    section.entries.parameters = map.next_value_seed(ParamList::<S>::new())?;
                }
                "consequences" => {
                    section.entries.consequences = map.next_value_seed(ConsList::<S>::new())?;
                }
                _ => section.regions = map.next_value_seed(Regions::<S>::new())?,
            }
        }
        Ok(section)
    }
}

/// One of a section's six keys, refused inside its own decoding (so at its position) otherwise.
struct SectionKey(&'static str);

impl<'de> Deserialize<'de> for SectionKey {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        const KEYS: [&str; 6] = [
            "extends",
            "default",
            "rules",
            "parameters",
            "consequences",
            "regions",
        ];
        let key = String::deserialize(d)?;
        KEYS.into_iter()
            .find(|known| *known == key)
            .map(SectionKey)
            .ok_or_else(|| {
                D::Error::custom(format!(
                    "'{key}' is not a key of a section ({})",
                    KEYS.join(", ")
                ))
            })
    }
}

/// An `extends` value: the name of one of the pack's reference lists whose chain is acyclic and short
/// enough, refused inside its own decoding so the refusal keeps its position.
struct Extends<S>(PhantomData<fn() -> S>);

impl<'de, S: InteractionSection> DeserializeSeed<'de> for Extends<S> {
    type Value = &'static str;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        let name = String::deserialize(d)?;
        Section::<S>::chain(&name).map_err(D::Error::custom)?;
        Ok(S::reference_lists()
            .into_iter()
            .map(|(list, _)| list)
            .find(|list| *list == name)
            .expect("the chain found it"))
    }
}

/// The selector a role key names, if `role` may be named here.
fn role_selector<'de, A: MapAccess<'de>>(
    map: &mut A,
    role: Role,
    allowed: &[Role],
    what: &dyn core::fmt::Display,
) -> Result<Selector, A::Error> {
    if !allowed.contains(&role) {
        let names: Vec<&str> = allowed.iter().map(|role| role.name()).collect();
        return Err(A::Error::custom(format!(
            "the role '{}' is not declared for {what} (its roles: {})",
            role.name(),
            names.join(", ")
        )));
    }
    map.next_value()
}

macro_rules! seq_seed {
    ($Seed:ident, $Item:ty, $entry:ident) => {
        struct $Seed<S> {
            in_region: bool,
            _pack: PhantomData<fn() -> S>,
        }

        impl<S> $Seed<S> {
            #[allow(dead_code)]
            const fn new_in(in_region: bool) -> Self {
                Self {
                    in_region,
                    _pack: PhantomData,
                }
            }
        }

        impl<'de, S: InteractionSection> DeserializeSeed<'de> for $Seed<S> {
            type Value = Vec<$Item>;

            fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
                d.deserialize_seq(self)
            }
        }

        impl<'de, S: InteractionSection> Visitor<'de> for $Seed<S> {
            type Value = Vec<$Item>;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a list of entries")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = seq.next_element_seed(EntrySeed::<S> {
                    in_region: self.in_region,
                    kind: EntryKind::$entry,
                    _pack: PhantomData,
                })? {
                    entries.push(entry.into());
                }
                Ok(entries)
            }
        }
    };
}

/// What an entry map decodes into before it is typed by its list.
enum Decoded<S: InteractionSection> {
    Rule(RuleEntry),
    Param(ParamEntry<Partial<S>>),
    Cons(ConsEntry<S::Knobs>),
}

impl<S: InteractionSection> From<Decoded<S>> for RuleEntry {
    fn from(decoded: Decoded<S>) -> Self {
        match decoded {
            Decoded::Rule(rule) => rule,
            _ => unreachable!("a rules list decodes rules"),
        }
    }
}

impl<S: InteractionSection> From<Decoded<S>> for ParamEntry<Partial<S>> {
    fn from(decoded: Decoded<S>) -> Self {
        match decoded {
            Decoded::Param(entry) => entry,
            _ => unreachable!("a parameters list decodes parameter entries"),
        }
    }
}

impl<S: InteractionSection> From<Decoded<S>> for ConsEntry<S::Knobs> {
    fn from(decoded: Decoded<S>) -> Self {
        match decoded {
            Decoded::Cons(entry) => entry,
            _ => unreachable!("a consequences list decodes consequence entries"),
        }
    }
}

seq_seed!(Rules, RuleEntry, Rule);
seq_seed!(ParamList, ParamEntry<Partial<S>>, Param);
seq_seed!(ConsList, ConsEntry<S::Knobs>, Cons);

impl<S> Rules<S> {
    const fn new(in_region: bool) -> Self {
        Self::new_in(in_region)
    }
}

impl<S> ParamList<S> {
    const fn new() -> Self {
        Self::new_in(false)
    }
}

impl<S> ConsList<S> {
    const fn new() -> Self {
        Self::new_in(false)
    }
}

#[derive(Clone, Copy)]
enum EntryKind {
    Rule,
    Param,
    Cons,
}

struct EntrySeed<S> {
    in_region: bool,
    kind: EntryKind,
    _pack: PhantomData<fn() -> S>,
}

impl<'de, S: InteractionSection> DeserializeSeed<'de> for EntrySeed<S> {
    type Value = Decoded<S>;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_map(self)
    }
}

/// `on`/`off`, or a YAML boolean.
struct OnOff(bool);

impl<'de> Deserialize<'de> for OnOff {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl Visitor<'_> for V {
            type Value = OnOff;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("on or off")
            }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<OnOff, E> {
                Ok(OnOff(v))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<OnOff, E> {
                match v {
                    "on" => Ok(OnOff(true)),
                    "off" => Ok(OnOff(false)),
                    other => Err(E::custom(format!("biography is on or off, not '{other}'"))),
                }
            }
        }
        d.deserialize_any(V)
    }
}

impl<'de, S: InteractionSection> Visitor<'de> for EntrySeed<S> {
    type Value = Decoded<S>;

    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("an entry: a map of roles and fields")
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        match self.kind {
            EntryKind::Rule => rule::<S, A>(map, self.in_region).map(Decoded::Rule),
            EntryKind::Param => param::<S, A>(map).map(Decoded::Param),
            EntryKind::Cons => cons::<S, A>(map).map(Decoded::Cons),
        }
    }
}

fn rule<'de, S: InteractionSection, A: MapAccess<'de>>(
    mut map: A,
    in_region: bool,
) -> Result<RuleEntry, A::Error> {
    let mut action = None;
    let mut effect = None;
    let mut selectors = Selectors::any();
    let mut named: Vec<Role> = Vec::new();
    while let Some(key) = map.next_key::<String>()? {
        match key.as_str() {
            "action" => {
                let id: String = map.next_value()?;
                let Some(declared) = S::ACTIONS.iter().find(|a| a.action.as_str() == id) else {
                    let known: Vec<&str> = S::ACTIONS.iter().map(|a| a.action.as_str()).collect();
                    return Err(A::Error::custom(format!(
                        "'{id}' is not an action '{}' declares (it declares: {})",
                        S::ID.as_str(),
                        if known.is_empty() {
                            "none".to_owned()
                        } else {
                            known.join(", ")
                        }
                    )));
                };
                if in_region && !declared.regional {
                    return Err(A::Error::custom(format!(
                        "a region may not carry a rule about '{id}': '{}' does not let a place \
                         scope it",
                        S::ID.as_str()
                    )));
                }
                action = Some(declared);
            }
            "effect" => effect = Some(map.next_value::<Effect>()?),
            other => match Role::named(other) {
                Some(role) => {
                    let selector: Selector = map.next_value()?;
                    selectors = selectors.with(role, selector);
                    named.push(role);
                }
                None => {
                    return Err(A::Error::custom(format!(
                        "'{other}' is not a key of a rule (action, effect, or a role)"
                    )));
                }
            },
        }
    }
    let action = action.ok_or_else(|| A::Error::custom("a rule names its action"))?;
    if let Some(role) = named.iter().find(|role| !action.roles.contains(role)) {
        let names: Vec<&str> = action.roles.iter().map(|role| role.name()).collect();
        return Err(A::Error::custom(format!(
            "the role '{}' is not declared for the action '{}' (its roles: {})",
            role.name(),
            action.action.as_str(),
            names.join(", ")
        )));
    }
    let effect =
        effect.ok_or_else(|| A::Error::custom("a rule states its effect: permit or forbid"))?;
    Ok(RuleEntry {
        action: action.action.clone(),
        selectors,
        effect,
    })
}

fn param<'de, S: InteractionSection, A: MapAccess<'de>>(
    mut map: A,
) -> Result<ParamEntry<Partial<S>>, A::Error> {
    let mut fields = Partial::<S>::default();
    let mut selectors = Selectors::any();
    let what = format!("the parameters of '{}'", S::ID.as_str());
    while let Some(key) = map.next_key::<String>()? {
        if let Some(role) = Role::named(&key) {
            let selector = role_selector(&mut map, role, S::PARAMETER_ROLES, &what)?;
            selectors = selectors.with(role, selector);
        } else if !fields.decode_field(&key, &mut map)? {
            return Err(A::Error::custom(format!(
                "'{key}' is neither a role nor a parameter of '{}' (its parameters: {})",
                S::ID.as_str(),
                <Partial<S> as Fields>::NAMES.join(", ")
            )));
        }
    }
    Ok(ParamEntry { selectors, fields })
}

fn cons<'de, S: InteractionSection, A: MapAccess<'de>>(
    mut map: A,
) -> Result<ConsEntry<S::Knobs>, A::Error> {
    let mut fact = None;
    let mut audience = None;
    let mut biography = None;
    let mut knobs = S::Knobs::default();
    let mut selectors = Selectors::any();
    let mut named: Vec<Role> = Vec::new();
    while let Some(key) = map.next_key::<String>()? {
        match key.as_str() {
            "fact" => {
                let id: String = map.next_value()?;
                let Some(declared) = S::FACTS.iter().find(|f| f.fact.as_str() == id) else {
                    let known: Vec<&str> = S::FACTS.iter().map(|f| f.fact.as_str()).collect();
                    return Err(A::Error::custom(format!(
                        "'{id}' is not a fact '{}' lets a list govern (it declares: {})",
                        S::ID.as_str(),
                        if known.is_empty() {
                            "none".to_owned()
                        } else {
                            known.join(", ")
                        }
                    )));
                };
                fact = Some(declared);
            }
            "audience" => audience = Some(map.next_value::<Audience>()?),
            "biography" => biography = Some(map.next_value::<OnOff>()?.0),
            other => match Role::named(other) {
                Some(role) => {
                    let selector: Selector = map.next_value()?;
                    selectors = selectors.with(role, selector);
                    named.push(role);
                }
                None => {
                    if !knobs.decode_field(other, &mut map)? {
                        return Err(A::Error::custom(format!(
                            "'{other}' is not a key of a consequence (fact, audience, biography, a \
                             role, or a knob of '{}')",
                            S::ID.as_str()
                        )));
                    }
                }
            },
        }
    }
    let fact = fact.ok_or_else(|| A::Error::custom("a consequence names its fact"))?;
    if let Some(role) = named
        .iter()
        .find(|role| !fact.roles.iter().any(|(declared, _)| declared == *role))
    {
        return Err(A::Error::custom(format!(
            "the role '{}' is not declared for the fact '{}'",
            role.name(),
            fact.fact.as_str()
        )));
    }
    if let Some(audience) = audience {
        if audience < fact.default_audience {
            return Err(A::Error::custom(format!(
                "the audience of '{}' may only narrow: {audience:?} is wider than its owner's \
                 {:?}",
                fact.fact.as_str(),
                fact.default_audience
            )));
        }
        if audience > fact.narrowest {
            return Err(A::Error::custom(format!(
                "the audience of '{}' may not be narrower than {:?}",
                fact.fact.as_str(),
                fact.narrowest
            )));
        }
    }
    if biography.is_some() && !fact.biography_configurable {
        return Err(A::Error::custom(format!(
            "whether '{}' is biographical is its owner's, and '{}' does not let a list choose",
            fact.fact.as_str(),
            S::ID.as_str()
        )));
    }
    Ok(ConsEntry {
        fact: fact.fact.clone(),
        selectors,
        audience,
        biography,
        knobs,
    })
}

struct Regions<S>(PhantomData<fn() -> S>);

impl<S> Regions<S> {
    const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'de, S: InteractionSection> DeserializeSeed<'de> for Regions<S> {
    type Value = BTreeMap<EntityKey, Entries<S>>;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_map(self)
    }
}

impl<'de, S: InteractionSection> Visitor<'de> for Regions<S> {
    type Value = BTreeMap<EntityKey, Entries<S>>;

    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("regions: a map of place keys to entries")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut regions = BTreeMap::new();
        while let Some(place) = map.next_key::<EntityKey>()? {
            let entries = map.next_value_seed(RegionSeed::<S>(PhantomData))?;
            if regions.insert(place.clone(), entries).is_some() {
                return Err(A::Error::custom(format!(
                    "the region '{place}' is listed twice"
                )));
            }
        }
        Ok(regions)
    }
}

struct RegionSeed<S>(PhantomData<fn() -> S>);

impl<'de, S: InteractionSection> DeserializeSeed<'de> for RegionSeed<S> {
    type Value = Entries<S>;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_map(self)
    }
}

impl<'de, S: InteractionSection> Visitor<'de> for RegionSeed<S> {
    type Value = Entries<S>;

    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("a region: rules, parameters and consequences")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut entries = Entries::<S>::default();
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "rules" => entries.rules = map.next_value_seed(Rules::<S>::new(true))?,
                "parameters" => entries.parameters = map.next_value_seed(ParamList::<S>::new())?,
                "consequences" => {
                    entries.consequences = map.next_value_seed(ConsList::<S>::new())?;
                }
                other => {
                    return Err(A::Error::custom(format!(
                        "'{other}' is not a key of a region (rules, parameters, consequences)"
                    )));
                }
            }
        }
        Ok(entries)
    }
}
