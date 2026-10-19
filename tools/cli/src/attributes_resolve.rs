//! `mineworld attributes resolve` and `mineworld attributes lock` (`docs/MODULE_SPEC.md` §8.1;
//! `pr-s30d-sampler.md` §§5.8, 5.9): the resolved cast S26 consumes (C-1 … C-7), and the one writer of a
//! world's lock. Reads the pack; `lock` writes `locks/<name>.yaml` and nothing else.

use std::collections::BTreeMap;
use std::path::Path;

use mineworld_attributes_model::lock::{Lock, LockEntry, Pinned};
use mineworld_attributes_model::sample::{Member, Origin, Resolution, Words, resolve as sample};
use mineworld_attributes_model::value::{Loose, decode};
use mineworld_attributes_model::{AttributesContext, Authored, Catalogue};
use mineworld_contracts::{EntityKey, Tags};
use mineworld_worldpack::format::SectionState;
use mineworld_worldpack::{Capability, PackRoots, WorldPack};
use serde_json::{Value, json};

use crate::described;

/// The world and its attributes context.
fn read(world: &Path, roots: &PackRoots) -> Result<(WorldPack, AttributesContext), String> {
    let pack = WorldPack::read_with(world, roots).map_err(described)?;
    let context = pack
        .context(Capability::Attributes)
        .and_then(|context| context.as_any().downcast_ref::<AttributesContext>())
        .cloned()
        .ok_or_else(|| {
            format!(
                "[mineworld] {} does not enable the attributes system",
                world.display()
            )
        })?;
    Ok((pack, context))
}

/// A person's decoded `attributes:` section.
fn authored<'p>(pack: &'p WorldPack, key: &EntityKey) -> Option<&'p Authored> {
    pack.people().get(key)?.sections.iter().find_map(|section| {
        match (&section.state, section.owner == Capability::Attributes) {
            (SectionState::Decoded(content), true) => content.as_any().downcast_ref::<Authored>(),
            _ => None,
        }
    })
}

/// What `resolve` was asked.
pub struct ResolveRequest<'a> {
    /// The World Pack.
    pub world: &'a Path,
    /// One JSON document.
    pub json: bool,
    /// Only this person.
    pub person: Option<&'a EntityKey>,
    /// Only values whose declaration carries this facet.
    pub facet: Option<&'a str>,
    /// A salt instead of the configuration's.
    pub salt: Option<u32>,
    /// Synthetic subjects instead of the world's people.
    pub synthetic: Option<u32>,
    /// The pack roots.
    pub roots: &'a PackRoots,
}

fn origin(origin: &Origin) -> Value {
    serde_json::to_value(origin).unwrap_or(Value::Null)
}

/// `mineworld attributes resolve`.
pub fn resolve(request: &ResolveRequest<'_>) -> Result<(), String> {
    let (pack, context) = read(request.world, request.roots)?;
    let catalogue = &context.catalogue;
    let population = context.population.as_ref();
    let seed = population
        .map(|population| population.seed.as_str())
        .unwrap_or_default();
    let salt = request
        .salt
        .unwrap_or_else(|| population.map_or(0, |population| population.salt));
    let words = Words::new(seed, salt);
    let profile = population.and_then(|population| population.profile.as_ref());
    let lock = population.and_then(|population| population.lock.as_ref());
    let empty = Tags::default();
    let (keys, synthetic): (Vec<String>, bool) = match request.synthetic {
        Some(n) => (
            (1..=n)
                .map(|index| format!("synthetic-{index:05}"))
                .collect(),
            true,
        ),
        None => (
            pack.people()
                .keys()
                .map(|key| key.as_str().to_owned())
                .collect(),
            false,
        ),
    };
    let members: Vec<Member<'_>> = keys
        .iter()
        .map(|key| {
            let entity = EntityKey::new(key).ok();
            let person = entity.as_ref().and_then(|entity| pack.people().get(entity));
            Member {
                key,
                tags: if synthetic {
                    &empty
                } else {
                    person.map_or(&empty, |person| &person.tags)
                },
                authored: if synthetic {
                    None
                } else {
                    entity.as_ref().and_then(|entity| authored(&pack, entity))
                },
            }
        })
        .collect();
    let resolution = sample(
        catalogue,
        profile,
        if synthetic { None } else { lock },
        words,
        &members,
    )
    .map_err(|refusal| format!("[mineworld] {refusal}"))?;
    let document = document(
        request,
        &pack,
        catalogue,
        &context,
        salt,
        &resolution,
        synthetic,
    );
    if request.json {
        println!("{document}");
    } else {
        print_text(&document);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn document(
    request: &ResolveRequest<'_>,
    pack: &WorldPack,
    catalogue: &Catalogue,
    context: &AttributesContext,
    salt: u32,
    resolution: &Resolution,
    synthetic: bool,
) -> Value {
    let population = context.population.as_ref();
    let profile = population.and_then(|population| population.profile.as_ref());
    let header = profile.map_or(Value::Null, |profile| {
        json!({
            "id": profile.id,
            "version": profile.version,
            "seed": population.map(|population| population.seed.clone()),
            "salt": salt,
            "covers": profile.covers,
        })
    });
    let mut sources = BTreeMap::new();
    let mut distributions = BTreeMap::new();
    let mut verify = Vec::new();
    if let Some(profile) = profile {
        for (name, source) in &profile.sources {
            sources.insert(
                name.clone(),
                json!({ "cite": source.cite, "status": source.status }),
            );
        }
        sources.insert("authored".to_owned(), json!({ "status": "authored" }));
        for (id, distribution) in &profile.distributions {
            if let Some(mark) = &distribution.verify {
                verify.push(json!({ "attribute": id, "verify": mark }));
            }
            distributions.insert(
                id.to_string(),
                json!({ "source": distribution.source, "verify": distribution.verify, "stratified": distribution.stratify }),
            );
        }
    }
    let mut document = json!({
        "schema": 1,
        "world": pack.id().to_string(),
        "profile": header,
        "sources": sources,
        "distributions": distributions,
        "support": resolution.support,
        "realised": resolution.realised,
        "verify": verify,
    });
    if synthetic {
        document["worst"] = worst(resolution);
        return document;
    }
    let mut people = BTreeMap::new();
    for (key, values) in &resolution.people {
        if request.person.is_some_and(|person| person.as_str() != key) {
            continue;
        }
        let listed: BTreeMap<String, Value> = values
            .iter()
            .filter(|(id, _)| {
                request.facet.is_none_or(|facet| {
                    catalogue
                        .get(id)
                        .is_some_and(|declaration| declaration.facets.contains(facet))
                })
            })
            .map(|(id, (value, from))| {
                (
                    id.to_string(),
                    json!({ "value": value, "origin": origin(from) }),
                )
            })
            .collect();
        people.insert(key.clone(), json!({ "look": null, "values": listed }));
    }
    document["people"] = json!(people);
    document
}

/// The largest marginal and conditional (groups ≥ 50) deviations, in percentage points with 2 decimals,
/// over stratified attributes.
fn worst(resolution: &Resolution) -> Value {
    let mut marginal: i128 = 0;
    let mut conditional: i128 = 0;
    for (id, groups) in &resolution.realised {
        if resolution.unstratified.contains(id) {
            continue;
        }
        let mut totals: BTreeMap<String, (i128, i128)> = BTreeMap::new();
        let mut all: i128 = 0;
        for realised in groups.values() {
            let n = i128::from(realised.n);
            all += n;
            for (value, count) in &realised.count {
                let target = realised
                    .target
                    .get(value)
                    .and_then(|text| mineworld_attributes_model::sample::micro::micro(text).ok())
                    .map_or(0, i128::from);
                let entry = totals.entry(value.to_string()).or_insert((0, 0));
                entry.0 += i128::from(*count) * 1_000_000;
                entry.1 += target;
                if n >= 50 {
                    let deviation =
                        (i128::from(*count) * 1_000_000 - target).abs() * 10_000 / (n * 1_000_000);
                    conditional = conditional.max(deviation);
                }
            }
        }
        if all > 0 {
            for (count, target) in totals.values() {
                marginal = marginal.max((count - target).abs() * 10_000 / (all * 1_000_000));
            }
        }
    }
    let pp = |basis: i128| format!("{}.{:02}", basis / 100, basis % 100);
    json!({ "marginal_pp": pp(marginal), "conditional_pp": pp(conditional) })
}

fn print_text(document: &Value) {
    println!("world {}", document["world"]);
    println!("profile {}", document["profile"]);
    if let Some(people) = document["people"].as_object() {
        for (key, person) in people {
            println!("{key}");
            for (id, entry) in person["values"].as_object().into_iter().flatten() {
                println!("  {id}  {}  {}", entry["value"], entry["origin"]);
            }
        }
    }
    if let Some(worst) = document.get("worst") {
        println!("worst {worst}");
    }
    for mark in document["verify"].as_array().into_iter().flatten() {
        println!("verify {mark}");
    }
}

/// What `lock` was asked.
pub struct LockRequest<'a> {
    /// The World Pack.
    pub world: &'a Path,
    /// The person.
    pub person: &'a EntityKey,
    /// The attribute.
    pub attribute: &'a str,
    /// Ruled or locked.
    pub origin: Pinned,
    /// `YYYY-MM-DD`.
    pub at: &'a str,
    /// By whom.
    pub by: Option<&'a str>,
    /// The value; absent: the value `resolve` gives now.
    pub value: Option<&'a str>,
    /// The pack roots.
    pub roots: &'a PackRoots,
}

/// `mineworld attributes lock`: writes or replaces one entry of the world's lock, canonically.
pub fn lock(request: &LockRequest<'_>) -> Result<(), String> {
    let (pack, context) = read(request.world, request.roots)?;
    let population = context.population.as_ref();
    let name = population
        .and_then(|population| population.lock_name.clone())
        .ok_or("[mineworld] name a lock in configure/attributes.yaml: lock: <name>")?;
    let catalogue = &context.catalogue;
    let declaration = catalogue
        .resolve(request.attribute)
        .map_err(|refusal| format!("[mineworld] {refusal}"))?;
    if !pack.people().contains_key(request.person) {
        return Err(format!(
            "[mineworld] UnknownPerson: the world declares no '{}'",
            request.person
        ));
    }
    let value = match request.value {
        Some(text) => {
            // A JSON scalar or list (1.660, true, ["a","b"]), else a bare slug or text.
            let loose: Loose =
                serde_json::from_str(text).unwrap_or_else(|_| Loose::Text(text.to_owned()));
            decode(&declaration.kind, &loose)
                .map_err(|error| format!("[mineworld] '{}': {error}", declaration.id))?
        }
        None => {
            let words = Words::new(
                population
                    .map(|population| population.seed.as_str())
                    .unwrap_or_default(),
                population.map_or(0, |population| population.salt),
            );
            let empty = Tags::default();
            let keys: Vec<String> = pack
                .people()
                .keys()
                .map(|key| key.as_str().to_owned())
                .collect();
            let members: Vec<Member<'_>> = keys
                .iter()
                .map(|key| {
                    let entity = EntityKey::new(key).ok();
                    Member {
                        key,
                        tags: entity
                            .as_ref()
                            .and_then(|entity| pack.people().get(entity))
                            .map_or(&empty, |person| &person.tags),
                        authored: entity.as_ref().and_then(|entity| authored(&pack, entity)),
                    }
                })
                .collect();
            let resolution = sample(
                catalogue,
                population.and_then(|population| population.profile.as_ref()),
                population.and_then(|population| population.lock.as_ref()),
                words,
                &members,
            )
            .map_err(|refusal| format!("[mineworld] {refusal}"))?;
            resolution
                .people
                .get(request.person.as_str())
                .and_then(|values| values.get(&declaration.id))
                .map(|(value, _)| value.clone())
                .ok_or_else(|| {
                    format!(
                        "[mineworld] '{}' resolves to no value of '{}'",
                        request.person, declaration.id
                    )
                })?
        }
    };
    let mut lock: Lock = population
        .and_then(|population| population.lock.clone())
        .unwrap_or_default();
    if let Some(profile) = population.and_then(|population| population.profile.as_ref()) {
        lock.profile = Some(mineworld_attributes_model::lock::LockedProfile {
            id: profile.id.clone(),
            version: profile.version,
        });
    }
    lock.seed = population
        .map(|population| population.seed.clone())
        .filter(|seed| !seed.is_empty());
    lock.people
        .entry(request.person.as_str().to_owned())
        .or_default()
        .insert(
            declaration.id.clone(),
            LockEntry {
                value,
                origin: request.origin,
                at: request.at.to_owned(),
                by: request.by.map(str::to_owned),
            },
        );
    let path = request.world.join("locks").join(format!("{name}.yaml"));
    std::fs::create_dir_all(request.world.join("locks"))
        .map_err(|error| format!("[mineworld] {error}"))?;
    std::fs::write(&path, lock.write())
        .map_err(|error| format!("[mineworld] {}: {error}", path.display()))?;
    println!(
        "{}: {} {} pinned ({})",
        path.display(),
        request.person,
        declaration.id,
        request.origin.as_str()
    );
    Ok(())
}
