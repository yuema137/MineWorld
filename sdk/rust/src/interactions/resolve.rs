//! Resolution: a section and the world's classes become one sorted value, refused at load when an
//! entry names an undefined class or two entries are ambiguous (`ARC-63` items 3 … 6). Pure: the same
//! files always resolve to the same value, whatever order their entries were written in.

use std::collections::{BTreeMap, BTreeSet};

use mineworld_authoring::{ClassName, ConfigurationRefusal, EntityClasses, EntryAt};
use mineworld_contracts::{ActionTypeId, EntityKey, EventTypeId};
use serde::{Deserialize, Serialize};

use super::InteractionSection;
use super::decl::{Audience, Effect};
use super::section::{
    ConsEntry, Entries, Fields, ParamEntry, Parameters, Partial, RuleEntry, Section,
};
use super::selector::{RoleSubjects, Selectors};

/// One place's resolved section — or the base, which applies wherever no region does: every level
/// merged, each vector sorted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Resolution<S: InteractionSection> {
    /// What an action no rule matches gets.
    pub default: Effect,
    /// The rules, by action then selectors.
    pub rules: Vec<RuleEntry>,
    /// The base parameters: the compiled defaults with every unscoped entry applied.
    pub parameters: S::Parameters,
    /// The scoped parameter entries, by selectors.
    pub scoped: Vec<ParamEntry<Partial<S>>>,
    /// The consequence entries, by fact then selectors.
    pub consequences: Vec<ConsEntry<S::Knobs>>,
}

/// What one consequence lookup answers before the owner's defaults fill it.
#[derive(Debug, Clone, PartialEq)]
pub struct Chosen<K> {
    /// The audience a list chose, if any.
    pub audience: Option<Audience>,
    /// The biographical flag a list chose, if any.
    pub biography: Option<bool>,
    /// The knobs, every field a list set.
    pub knobs: K,
}

impl<S: InteractionSection> Resolution<S> {
    /// What the compiled default resolves to: no rule, the default parameters, no consequence.
    pub fn compiled() -> Self {
        Self {
            default: Effect::Permit,
            rules: Vec::new(),
            parameters: S::Parameters::default(),
            scoped: Vec::new(),
            consequences: Vec::new(),
        }
    }

    /// Whether `action` is permitted for these subjects: the most specific matching rules decide, a
    /// `forbid` among them wins, and with no matching rule the default applies.
    pub fn permits(
        &self,
        classes: &EntityClasses,
        action: &ActionTypeId,
        subjects: &RoleSubjects<'_>,
    ) -> bool {
        let start = self.rules.partition_point(|rule| rule.action < *action);
        let mut best: Option<(usize, Effect)> = None;
        for rule in self.rules[start..]
            .iter()
            .take_while(|rule| rule.action == *action)
            .filter(|rule| rule.selectors.matches(classes, subjects))
        {
            let specificity = rule.selectors.specificity();
            best = match best {
                Some((s, effect)) if s > specificity => Some((s, effect)),
                Some((s, effect)) if s == specificity => Some((s, effect.max(rule.effect))),
                _ => Some((specificity, rule.effect)),
            };
        }
        best.map_or(self.default, |(_, effect)| effect) == Effect::Permit
    }

    /// The parameters for these subjects: the base, then every matching scoped entry, least specific
    /// first, field by field.
    pub fn parameters(
        &self,
        classes: &EntityClasses,
        subjects: &RoleSubjects<'_>,
    ) -> S::Parameters {
        let mut matching: Vec<&ParamEntry<Partial<S>>> = self
            .scoped
            .iter()
            .filter(|entry| entry.selectors.matches(classes, subjects))
            .collect();
        matching.sort_by_key(|entry| entry.selectors.specificity());
        let mut parameters = self.parameters.clone();
        for entry in matching {
            parameters.apply(&entry.fields);
        }
        parameters
    }

    /// What the list chooses for `fact` with these subjects: every matching entry, least specific
    /// first, field by field.
    pub fn consequence(
        &self,
        classes: &EntityClasses,
        fact: &EventTypeId,
        subjects: &RoleSubjects<'_>,
    ) -> Chosen<S::Knobs> {
        let start = self
            .consequences
            .partition_point(|entry| entry.fact < *fact);
        let mut matching: Vec<&ConsEntry<S::Knobs>> = self.consequences[start..]
            .iter()
            .take_while(|entry| entry.fact == *fact)
            .filter(|entry| entry.selectors.matches(classes, subjects))
            .collect();
        matching.sort_by_key(|entry| entry.selectors.specificity());
        let mut chosen = Chosen {
            audience: None,
            biography: None,
            knobs: S::Knobs::default(),
        };
        for entry in matching {
            chosen.audience = entry.audience.or(chosen.audience);
            chosen.biography = entry.biography.or(chosen.biography);
            chosen.knobs.overlay(&entry.knobs);
        }
        chosen
    }
}

/// A configured section, resolved: the classes it can be affected by, the base, and each region.
/// What `<pack>-interactions-configured` states.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Resolved<S: InteractionSection> {
    /// The class definitions its selectors can be decided by (`ARC-64` item 3).
    pub classes: EntityClasses,
    /// Wherever no region applies.
    pub base: Resolution<S>,
    /// Each region, by place key, sorted.
    pub regions: Vec<(EntityKey, Resolution<S>)>,
}

impl<S: InteractionSection> Resolved<S> {
    /// The resolution that applies at `place`: its region's, or the base.
    pub fn at(&self, place: Option<&EntityKey>) -> &Resolution<S> {
        place
            .and_then(|place| {
                self.regions
                    .binary_search_by(|(key, _)| key.cmp(place))
                    .ok()
                    .map(|index| &self.regions[index].1)
            })
            .unwrap_or(&self.base)
    }
}

/// One level: where its entries are written (a list prefix) and the entries.
struct Level<'a, S: InteractionSection> {
    prefix: String,
    entries: &'a Entries<S>,
}

fn at(prefix: &str, list: &str, index: usize) -> EntryAt {
    EntryAt {
        list: format!("{prefix}{list}"),
        index,
    }
}

/// The fields of a consequence entry a lookup merges, compared for ambiguity.
fn cons_conflict<K: Fields>(a: &ConsEntry<K>, b: &ConsEntry<K>) -> Option<String> {
    if let (Some(x), Some(y)) = (a.audience, b.audience)
        && x != y
    {
        return Some("audience".to_owned());
    }
    if let (Some(x), Some(y)) = (a.biography, b.biography)
        && x != y
    {
        return Some("biography".to_owned());
    }
    a.knobs.conflict(&b.knobs).map(str::to_owned)
}

fn cons_overlay<K: Fields>(into: &mut ConsEntry<K>, from: &ConsEntry<K>) {
    into.audience = from.audience.or(into.audience);
    into.biography = from.biography.or(into.biography);
    into.knobs.overlay(&from.knobs);
}

/// Every class the section's entries name.
fn named_classes<S: InteractionSection>(entries: &Entries<S>) -> Vec<(&ClassName, EntryAt)> {
    let rules = entries
        .rules
        .iter()
        .enumerate()
        .flat_map(|(i, e)| e.selectors.classes().map(move |c| (c, ("rules", i))));
    let parameters = entries
        .parameters
        .iter()
        .enumerate()
        .flat_map(|(i, e)| e.selectors.classes().map(move |c| (c, ("parameters", i))));
    let consequences = entries
        .consequences
        .iter()
        .enumerate()
        .flat_map(|(i, e)| e.selectors.classes().map(move |c| (c, ("consequences", i))));
    rules
        .chain(parameters)
        .chain(consequences)
        .map(|(class, (list, index))| (class, at("", list, index)))
        .collect()
}

/// Resolves `section` against `classes` (`ARC-63` items 4 and 5).
///
/// # Errors
///
/// [`ConfigurationRefusal::ClassUndefined`] for a selector naming a class that is neither defined nor
/// implicit; [`ConfigurationRefusal::Ambiguous`] for two entries of equal specificity that overlap and
/// disagree on a field.
pub fn resolve<S: InteractionSection>(
    section: &Section<S>,
    classes: &EntityClasses,
) -> Result<Resolved<S>, ConfigurationRefusal> {
    let mut referenced = BTreeSet::new();
    let regions = section
        .regions
        .iter()
        .map(|(place, entries)| (format!("regions.{place}."), entries));
    for (prefix, entries) in std::iter::once((String::new(), &section.entries)).chain(regions) {
        for (class, mut entry) in named_classes(entries) {
            if classes.type_of(class).is_none() {
                entry.list = format!("{prefix}{}", entry.list);
                return Err(ConfigurationRefusal::ClassUndefined {
                    class: class.clone(),
                    entry,
                });
            }
            referenced.insert(class.clone());
        }
    }

    let lists = S::reference_lists();
    let default_list = lists
        .iter()
        .find(|(name, _)| *name == "default")
        .map(|(_, list)| list.clone())
        .unwrap_or_default();
    let chain = match section.extends {
        Some(name) => Section::<S>::chain(name).unwrap_or_default(),
        None => Vec::new(),
    };
    let mut levels = vec![Level {
        prefix: "reference list 'default' ".to_owned(),
        entries: &default_list.entries,
    }];
    for (name, list) in &chain {
        levels.push(Level {
            prefix: format!("reference list '{name}' "),
            entries: &list.entries,
        });
    }
    levels.push(Level {
        prefix: String::new(),
        entries: &section.entries,
    });
    let default = section
        .default
        .or_else(|| chain.iter().rev().find_map(|(_, list)| list.default))
        .or(default_list.default)
        .unwrap_or(Effect::Permit);

    let base = resolve_levels(&levels, default, classes)?;
    let mut resolved_regions = Vec::with_capacity(section.regions.len());
    for (place, entries) in &section.regions {
        let mut with_region: Vec<Level<'_, S>> = levels
            .iter()
            .map(|level| Level {
                prefix: level.prefix.clone(),
                entries: level.entries,
            })
            .collect();
        with_region.push(Level {
            prefix: format!("regions.{place}."),
            entries,
        });
        resolved_regions.push((
            place.clone(),
            resolve_levels(&with_region, default, classes)?,
        ));
    }
    Ok(Resolved {
        classes: classes.restricted_to(&referenced),
        base,
        regions: resolved_regions,
    })
}

type ParamMap<P> = BTreeMap<Selectors, (P, EntryAt)>;
type ConsMap<K> = BTreeMap<(EventTypeId, Selectors), (ConsEntry<K>, EntryAt)>;

fn resolve_levels<S: InteractionSection>(
    levels: &[Level<'_, S>],
    default: Effect,
    classes: &EntityClasses,
) -> Result<Resolution<S>, ConfigurationRefusal> {
    let mut rules: BTreeMap<(ActionTypeId, Selectors), Effect> = BTreeMap::new();
    let mut parameters: ParamMap<Partial<S>> = BTreeMap::new();
    let mut consequences: ConsMap<S::Knobs> = BTreeMap::new();
    for level in levels {
        // Within one level, entries with the same key combine: forbid wins between rules, and two
        // parameter or consequence entries disagreeing on a field are ambiguous.
        let mut level_rules: BTreeMap<(ActionTypeId, Selectors), Effect> = BTreeMap::new();
        for rule in &level.entries.rules {
            level_rules
                .entry((rule.action.clone(), rule.selectors.clone()))
                .and_modify(|effect| *effect = (*effect).max(rule.effect))
                .or_insert(rule.effect);
        }
        rules.extend(level_rules);

        let mut level_params: ParamMap<Partial<S>> = BTreeMap::new();
        for (index, entry) in level.entries.parameters.iter().enumerate() {
            let here = at(&level.prefix, "parameters", index);
            if let Some((held, first)) = level_params.get_mut(&entry.selectors) {
                if let Some(field) = held.conflict(&entry.fields) {
                    return Err(ambiguous(first, &here, field));
                }
                held.overlay(&entry.fields);
            } else {
                level_params.insert(entry.selectors.clone(), (entry.fields.clone(), here));
            }
        }
        for (selectors, (fields, origin)) in level_params {
            match parameters.get_mut(&selectors) {
                Some((held, at)) => {
                    held.overlay(&fields);
                    *at = origin;
                }
                None => {
                    parameters.insert(selectors, (fields, origin));
                }
            }
        }

        let mut level_cons: ConsMap<S::Knobs> = BTreeMap::new();
        for (index, entry) in level.entries.consequences.iter().enumerate() {
            let here = at(&level.prefix, "consequences", index);
            let key = (entry.fact.clone(), entry.selectors.clone());
            if let Some((held, first)) = level_cons.get_mut(&key) {
                if let Some(field) = cons_conflict(held, entry) {
                    return Err(ambiguous(first, &here, &field));
                }
                cons_overlay(held, entry);
            } else {
                level_cons.insert(key, (entry.clone(), here));
            }
        }
        for (key, (entry, origin)) in level_cons {
            match consequences.get_mut(&key) {
                Some((held, at)) => {
                    cons_overlay(held, &entry);
                    *at = origin;
                }
                None => {
                    consequences.insert(key, (entry, origin));
                }
            }
        }
    }

    // Across different selectors: equal specificity, overlapping, disagreeing — refused.
    let scoped: Vec<(&Selectors, &(Partial<S>, EntryAt))> = parameters
        .iter()
        .filter(|(selectors, _)| selectors.specificity() > 0)
        .collect();
    for (i, (a, (fa, oa))) in scoped.iter().enumerate() {
        for (b, (fb, ob)) in &scoped[i + 1..] {
            if a.specificity() == b.specificity()
                && a.overlaps(b, classes)
                && let Some(field) = fa.conflict(fb)
            {
                return Err(ambiguous(oa, ob, field));
            }
        }
    }
    let held: Vec<_> = consequences.iter().collect();
    for (i, ((fact_a, a), (ea, oa))) in held.iter().enumerate() {
        for ((fact_b, b), (eb, ob)) in &held[i + 1..] {
            if fact_a == fact_b
                && a.specificity() == b.specificity()
                && a.overlaps(b, classes)
                && let Some(field) = cons_conflict(ea, eb)
            {
                return Err(ambiguous(oa, ob, &field));
            }
        }
    }

    let mut base = S::Parameters::default();
    if let Some((fields, _)) = parameters.get(&Selectors::any()) {
        base.apply(fields);
    }
    Ok(Resolution {
        default,
        rules: rules
            .into_iter()
            .map(|((action, selectors), effect)| RuleEntry {
                action,
                selectors,
                effect,
            })
            .collect(),
        parameters: base,
        scoped: parameters
            .into_iter()
            .filter(|(selectors, _)| selectors.specificity() > 0)
            .map(|(selectors, (fields, _))| ParamEntry { selectors, fields })
            .collect(),
        consequences: consequences.into_values().map(|(entry, _)| entry).collect(),
    })
}

fn ambiguous(first: &EntryAt, second: &EntryAt, field: &str) -> ConfigurationRefusal {
    let (first, second) = if first <= second {
        (first.clone(), second.clone())
    } else {
        (second.clone(), first.clone())
    };
    ConfigurationRefusal::Ambiguous {
        first,
        second,
        field: field.to_owned(),
    }
}
