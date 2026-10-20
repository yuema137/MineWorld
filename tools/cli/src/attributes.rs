//! `mineworld attributes catalogue | show | check` (`docs/MODULE_SPEC.md` §8.1; S30-c): what a world's
//! attribute catalogue holds, what each person resolves to, and whether it is clean. Reads the pack
//! only; nothing printed here reaches a fact.

use std::collections::BTreeMap;
use std::path::Path;

use mineworld_attributes_model::backend;
use mineworld_attributes_model::catalogue::CORE;
use mineworld_attributes_model::render::{self, View};
use mineworld_attributes_model::section::Restriction;
use mineworld_attributes_model::{
    ATTRIBUTES, AttributeValue, AttributesContext, Authored, Catalogue, Declaration, Mutability,
    Traits, derive,
};
use mineworld_contracts::{EntityKey, WorldTime};
use mineworld_worldpack::format::SectionState;
use mineworld_worldpack::{Capability, PackRoots, WorldPack};
use serde_json::{Value, json};

use crate::described;

/// The world, read, and its attributes context: refused when the world does not enable `attributes`.
fn read_context(world: &Path, roots: &PackRoots) -> Result<(WorldPack, AttributesContext), String> {
    let pack = WorldPack::read_with(world, roots).map_err(described)?;
    let context = pack
        .context(Capability::Attributes)
        .and_then(|context| context.as_any().downcast_ref::<AttributesContext>())
        .cloned()
        .ok_or_else(|| {
            format!(
                "[mineworld] {} does not enable the attributes system, so it has no attribute \
                 catalogue",
                world.display()
            )
        })?;
    Ok((pack, context))
}

/// The world, read, and its catalogue.
fn read(world: &Path, roots: &PackRoots) -> Result<(WorldPack, Catalogue), String> {
    read_context(world, roots).map(|(pack, context)| (pack, context.catalogue))
}

/// The header lines naming the backend and the mask, when the world's packs provide a backend.
fn composition_lines(context: &AttributesContext) -> Vec<String> {
    context
        .composition
        .as_ref()
        .map(|composed| {
            vec![
                format!(
                    "backend: {} {}",
                    composed.backend.id, composed.backend.version
                ),
                format!("mask: {} {}", composed.mask.id, composed.mask.version),
            ]
        })
        .unwrap_or_default()
}

/// The built-in catalogue check (§5.6), when the world uses the `mineworld` backend: `Ok(None)` when it
/// does not apply.
fn core_check(context: &AttributesContext) -> Result<Option<usize>, Vec<String>> {
    let Some(composed) = context
        .composition
        .as_ref()
        .filter(|composed| composed.backend.id == backend::MINEWORLD)
    else {
        return Ok(None);
    };
    let Some(default_mask) = composed.default_mask.as_ref() else {
        return Err(vec![format!(
            "the backend '{}' ships no default mask '{}'",
            composed.backend.id, composed.backend.masks.default
        )]);
    };
    let core = context
        .catalogue
        .attributes()
        .map(|declaration| &declaration.id)
        .chain(context.catalogue.removed().keys())
        .filter(|id| id.namespace() == CORE);
    backend::check_core(core, default_mask).map(Some)
}

/// `mineworld attributes backends`: the backends the world's packs provide, and the one it uses.
pub fn backends(world: &Path, roots: &PackRoots) -> Result<(), String> {
    let (_, context) = read_context(world, roots)?;
    if context.backends.is_empty() {
        println!("no backend: the world's catalogue is every collected declaration");
        return Ok(());
    }
    let used = context
        .composition
        .as_ref()
        .map(|composed| composed.backend.id.as_str());
    for backend in &context.backends {
        println!(
            "{}{} {} (pack {}, licence {}): {}; masks {:?}, default {}; sampling {:?}; provenance {:?}",
            if used == Some(backend.id.as_str()) {
                "* "
            } else {
                "  "
            },
            backend.id,
            backend.version,
            backend.pack,
            backend.license,
            backend.attribution,
            backend.masks.ships,
            backend.masks.default,
            backend.sampling,
            backend.provenance,
        );
    }
    for line in composition_lines(&context) {
        println!("{line}");
    }
    Ok(())
}

/// One declaration as `catalogue --json` prints it.
fn declared(catalogue: &Catalogue, declaration: &Declaration) -> Value {
    let mut value = serde_json::to_value(declaration).unwrap_or(Value::Null);
    if let (Value::Object(map), Some(origin)) = (&mut value, catalogue.origin(&declaration.id)) {
        map.insert(
            "origin".to_owned(),
            Value::String(origin.display().to_string().replace('\\', "/")),
        );
    }
    value
}

/// How many declarations each namespace holds.
fn per_namespace(catalogue: &Catalogue) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for declaration in catalogue.attributes() {
        *counts
            .entry(declaration.id.namespace().to_owned())
            .or_insert(0) += 1;
    }
    counts
}

/// `mineworld attributes catalogue`.
pub fn catalogue(
    world: &Path,
    json: bool,
    facet: Option<&str>,
    roots: &PackRoots,
) -> Result<(), String> {
    let (_, context) = read_context(world, roots)?;
    let catalogue = &context.catalogue;
    let selected: Vec<&Declaration> = catalogue
        .attributes()
        .filter(|declaration| facet.is_none_or(|facet| declaration.facets.contains(facet)))
        .collect();
    if json {
        let attributes: Vec<Value> = selected
            .iter()
            .map(|declaration| declared(catalogue, declaration))
            .collect();
        let composition = context.composition.as_ref();
        let document = json!({
            "attributes": attributes,
            "namespaces": per_namespace(catalogue),
            "backend": composition.map(|composed| json!({ "id": composed.backend.id, "version": composed.backend.version })),
            "mask": composition.map(|composed| json!({ "id": composed.mask.id, "version": composed.mask.version })),
        });
        println!("{document}");
        return Ok(());
    }
    for line in composition_lines(&context) {
        println!("{line}");
    }
    for declaration in &selected {
        println!(
            "{}  {}  {}  {}  {}  owner {}  {:?}  {:?}  {}",
            declaration.id,
            declaration.group,
            declaration.kind.domain(),
            declaration.visibility,
            declaration.mutability,
            declaration.owner,
            declaration.label,
            declaration.phrase,
            catalogue
                .origin(&declaration.id)
                .map(|origin| origin.display().to_string().replace('\\', "/"))
                .unwrap_or_default(),
        );
    }
    for (namespace, count) in per_namespace(catalogue) {
        println!("{namespace}: {count} attribute(s)");
    }
    Ok(())
}

/// A person's decoded `attributes:` section, if their file has one.
fn authored<'p>(pack: &'p WorldPack, key: &EntityKey) -> Option<&'p Authored> {
    pack.people().get(key)?.sections.iter().find_map(|section| {
        match (&section.state, section.owner == Capability::Attributes) {
            (SectionState::Decoded(content), true) => content.as_any().downcast_ref::<Authored>(),
            _ => None,
        }
    })
}

/// Each resolved value of one person with its origin: `authored` or `default` (S30-c's precedence).
fn resolved<'c>(
    catalogue: &'c Catalogue,
    authored: Option<&Authored>,
) -> Vec<(&'c Declaration, AttributeValue, &'static str)> {
    catalogue
        .attributes()
        .filter(|declaration| declaration.mutability != Mutability::Derived)
        .filter_map(|declaration| {
            if let Some(value) = authored.and_then(|authored| authored.values.get(&declaration.id))
            {
                return Some((declaration, value.clone(), "authored"));
            }
            declaration
                .default
                .clone()
                .map(|value| (declaration, value, "default"))
        })
        .collect()
}

/// The rendered lines of one view at the epoch.
fn view(
    catalogue: &Catalogue,
    values: &[(&Declaration, AttributeValue, &'static str)],
    view: View,
) -> Vec<Value> {
    let plain = values
        .iter()
        .filter(|(declaration, ..)| declaration.owner == ATTRIBUTES)
        .map(|(declaration, value, _)| (declaration.id.clone(), value.clone()));
    let traits = Traits::stated(catalogue, plain).unwrap_or_default();
    let owned: BTreeMap<_, _> = values
        .iter()
        .map(|(declaration, value, _)| (declaration.id.clone(), value.clone()))
        .collect();
    catalogue
        .attributes()
        .filter_map(|declaration| {
            let value = match declaration.mutability {
                Mutability::Derived => {
                    derive::value(catalogue, declaration, &traits, WorldTime::EPOCH)
                }
                _ => owned.get(&declaration.id).cloned(),
            }?;
            render::entry(declaration, &value, view)
        })
        .filter_map(|entry| serde_json::to_value(entry).ok())
        .collect()
}

/// `mineworld attributes show`.
pub fn show(
    world: &Path,
    person: Option<&EntityKey>,
    json: bool,
    roots: &PackRoots,
) -> Result<(), String> {
    let (pack, catalogue) = read(world, roots)?;
    if let Some(person) = person
        && !pack.people().contains_key(person)
    {
        return Err(format!(
            "[mineworld] {} declares no person '{person}'",
            world.display()
        ));
    }
    let mut people = Vec::new();
    for key in pack
        .people()
        .keys()
        .filter(|key| person.is_none_or(|person| person == *key))
    {
        let values = resolved(&catalogue, authored(&pack, key));
        let listed: Vec<Value> = values
            .iter()
            .map(|(declaration, value, origin)| {
                json!({ "attribute": declaration.id, "value": value, "origin": origin })
            })
            .collect();
        people.push(json!({
            "person": key.as_str(),
            "values": listed,
            "holder": view(&catalogue, &values, View::Holder),
            "other": view(&catalogue, &values, View::Other),
        }));
    }
    if json {
        println!("{}", json!({ "people": people }));
        return Ok(());
    }
    for person in &people {
        println!("{}", person["person"].as_str().unwrap_or_default());
        for value in person["values"].as_array().into_iter().flatten() {
            println!(
                "  {}  {}  {}",
                value["attribute"], value["value"], value["origin"]
            );
        }
        for (title, key) in [("holder", "holder"), ("another observer", "other")] {
            let texts: Vec<&str> = person[key]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|entry| entry["text"].as_str())
                .collect();
            println!("  {title}: {}", texts.join("; "));
        }
    }
    Ok(())
}

/// `mineworld attributes check`: everything `catalogue` validates (a refusal exits 1), then restrictions
/// no profile draws, counts per namespace, and every pack's `uses:`.
pub fn check(world: &Path, roots: &PackRoots) -> Result<(), String> {
    let (pack, context) = read_context(world, roots)?;
    let catalogue = &context.catalogue;
    for line in composition_lines(&context) {
        println!("{line}");
    }
    for key in pack.people().keys() {
        let Some(authored) = authored(&pack, key) else {
            continue;
        };
        for (id, restriction) in &authored.restrictions {
            if !authored.values.contains_key(id) {
                let kind = match restriction {
                    Restriction::Allowed(_) => "allowed",
                    Restriction::Between(..) => "between",
                };
                println!(
                    "{key}: {id} is restricted ({kind}) and no profile draws it: restricted, never drawn"
                );
            }
        }
    }
    for (namespace, count) in per_namespace(catalogue) {
        println!("{namespace}: {count} attribute(s)");
    }
    let profile = context
        .population
        .as_ref()
        .and_then(|population| population.profile.as_ref());
    for (id, distribution) in profile
        .into_iter()
        .flat_map(|profile| &profile.distributions)
    {
        if let Some(mark) = &distribution.verify {
            println!("verify: {id}: {mark}");
        }
    }
    match core_check(&context) {
        Ok(Some(count)) => {
            println!("core: {count} attribute(s), each with its reader line");
            if let Some(mask) = context
                .composition
                .as_ref()
                .and_then(|c| c.default_mask.as_ref())
            {
                for id in catalogue
                    .attributes()
                    .map(|declaration| &declaration.id)
                    .filter(|id| id.namespace() == CORE)
                {
                    println!("  {id}: {}", mask.why_of(id).unwrap_or_default());
                }
            }
        }
        Ok(None) => {}
        Err(refusals) => {
            return Err(refusals
                .iter()
                .map(|refusal| format!("[mineworld] {refusal}"))
                .collect::<Vec<_>>()
                .join("\n"));
        }
    }
    for (pack, uses) in catalogue.uses() {
        let reads: Vec<String> = uses
            .reads
            .iter()
            .map(|(id, read)| format!("{id} ({})", read.kind))
            .collect();
        let proposes: Vec<String> = uses.proposes.iter().map(ToString::to_string).collect();
        println!(
            "{pack}: reads [{}], proposes [{}]",
            reads.join(", "),
            proposes.join(", ")
        );
    }
    println!("{} attribute(s); clean", catalogue.len());
    Ok(())
}
