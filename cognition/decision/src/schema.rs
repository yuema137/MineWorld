//! `decisions.yaml` as JSON Schema, for the generated reference (`docs/DECISIONS.md` `ARC-90`;
//! `docs/MODULE_SPEC.md` §4.1; step-25 §10.3 PR 25a C6).
//!
//! The file is read by a hand-written positioned pass (`read.rs`), not a derive, so its schema is
//! written here beside it, and a test holds its keys equal to the pass's own key lists. Which models
//! exist, and each in-process model's parameters, come from the build's [`Catalog`]: the page names
//! models, never a backend, a provider or a key.

use schemars::{Schema, SchemaGenerator};
use serde_json::{Map, Value, json};

use crate::format::PolicyId;
use crate::models::Catalog;

/// The decision list of a build whose installed models are `C`.
pub fn file_schema<C: Catalog>() -> Schema {
    let mut generator = SchemaGenerator::default();
    let ids: Vec<&str> = C::AVAILABLE.iter().map(|model| model.id()).collect();
    let mut chained = ids.clone();
    chained.push("none");
    let mut models = Map::new();
    for model in C::AVAILABLE {
        models.insert(
            model.id().to_owned(),
            entry(*model, &mut generator, &chained),
        );
    }
    let example_model = C::AVAILABLE
        .iter()
        .find(|model| !model.is_external())
        .map_or("rule", |model| model.id());
    let policy = format!("^[a-z][a-z0-9-]{{0,{}}}$", PolicyId::MAX_BYTES - 1);
    let mut schema = schemars::json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "DecisionList",
        "description": "Which decision model drives each seat of the world, and which memory policy \
                        each seat's controller uses. Every key is optional; with no file every seat is \
                        driven by the rule model.",
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "models": {
                "description": "The models this file configures, each once, by id: an in-process \
                                model's parameters, an out-of-process model's fallback. A model used \
                                but not listed runs with its defaults.",
                "type": "object",
                "additionalProperties": false,
                "properties": Value::Object(models),
            },
            "default": {
                "description": "The model that drives every seat no assignment names. Absent, the \
                                rule model.",
                "type": "string",
                "enum": ids,
            },
            "assign": {
                "description": "Which model drives a seat, or every seat of a class. A seat's entry \
                                wins over its class's, and a class's over the default; one entry per \
                                seat and one per class.",
                "type": "array",
                "items": { "$ref": "#/$defs/Assignment" },
            },
            "memory": {
                "description": "Which memory policy each seat's controller uses. Absent, the \
                                cognition process's own default for every seat.",
                "$ref": "#/$defs/Memory",
            },
        },
        "$defs": {
            "Assignment": {
                "description": "One assignment: exactly one of seat and class, and the model.",
                "type": "object",
                "additionalProperties": false,
                "required": ["model"],
                "properties": {
                    "seat": {
                        "description": "A person of the world listed under seats: in world.yaml.",
                        "type": "string",
                    },
                    "class": {
                        "description": "A class of persons defined in configure/classes.yaml; not \
                                        person, which default: covers.",
                        "type": "string",
                    },
                    "model": {
                        "description": "The model that drives it.",
                        "type": "string",
                        "enum": ids,
                    },
                },
            },
            "Memory": {
                "description": "Which memory policy each seat's controller uses. Passed to the \
                                cognition process as written; whether a policy exists is its to judge.",
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "default": {
                        "description": "The policy of every seat no memory assignment names. \
                                        Absent, the cognition process's own default.",
                        "$ref": "#/$defs/MemoryPolicy",
                    },
                    "assign": {
                        "description": "Which policy a seat, or every seat of a class, uses; one \
                                        entry per seat and one per class.",
                        "type": "array",
                        "items": { "$ref": "#/$defs/MemoryAssignment" },
                    },
                },
            },
            "MemoryPolicy": {
                "description": "A memory policy, with its fallback and parameters.",
                "type": "object",
                "additionalProperties": false,
                "required": ["policy"],
                "properties": memory_fields(&policy, false),
            },
            "MemoryAssignment": {
                "description": "One memory assignment: exactly one of seat and class, and the policy.",
                "type": "object",
                "additionalProperties": false,
                "required": ["policy"],
                "properties": memory_fields(&policy, true),
            },
        },
        "examples": [{
            "models": { example_model: {} },
            "default": example_model,
            "assign": [{ "seat": "alice", "model": example_model }],
            "memory": {
                "default": { "policy": "consolidate" },
                "assign": [{
                    "seat": "alice",
                    "policy": "lm-summarise",
                    "fallback": "consolidate",
                    "parameters": { "calls_per_sim_day": 4 },
                }],
            },
        }],
    });
    // A model's parameters may refer to their own nested types (`MotivatedParameters` refers to `Base`):
    // those definitions live in the generator, so they are carried into this file's `$defs`.
    let definitions = generator.take_definitions(true);
    if let Some(defs) = schema
        .as_object_mut()
        .and_then(|root| root.get_mut("$defs"))
        .and_then(Value::as_object_mut)
    {
        defs.extend(definitions);
    }
    schema
}

/// One installed model's entry under `models:`.
fn entry<C: Catalog>(model: C, generator: &mut SchemaGenerator, chained: &[&str]) -> Value {
    let id = model.id();
    match model.parameters_schema(generator) {
        Some(parameters) => {
            let mut parameters = parameters.to_value();
            if let Value::Object(map) = &mut parameters {
                map.insert(
                    "description".to_owned(),
                    Value::String(format!(
                        "The {id} model's parameters; a key it does not take, or a value outside \
                         its bound, is refused. An empty mapping when it takes none."
                    )),
                );
            }
            json!({
                "description": format!("The {id} model, which the host runs in process."),
                "type": "object",
                "additionalProperties": false,
                "properties": { "parameters": parameters },
            })
        }
        None => json!({
            "description": format!(
                "The {id} model, which runs out of process; it must name a fallback."
            ),
            "type": "object",
            "additionalProperties": false,
            "required": ["fallback"],
            "properties": {
                "fallback": {
                    "description": "The model that drives the seat while this one is absent or out \
                                    of budget, or none to leave the seat idle; at most four steps, \
                                    no cycle.",
                    "type": "string",
                    "enum": chained,
                },
            },
        }),
    }
}

/// A memory entry's fields, with `seat` and `class` for an assignment.
fn memory_fields(policy: &str, assignment: bool) -> Value {
    let mut fields = json!({
        "policy": {
            "description": "The memory policy's id.",
            "type": "string",
            "pattern": policy,
        },
        "fallback": {
            "description": "The policy used when this one cannot run.",
            "type": "string",
            "pattern": policy,
        },
        "parameters": {
            "description": "The policy's parameters, passed through as written: a mapping with text \
                            keys and no YAML tags.",
            "type": "object",
        },
        "store_bound_mib": {
            "description": "The per-person memory store's bound, in MiB, which a world may raise above \
                            the cognition host's floor of 20 MiB. Absent, the floor. Passed through \
                            as written; the cognition host judges it.",
            "type": "integer",
            "minimum": 20,
            "maximum": 1_024,
        },
        "scopes": {
            "description": "The person's memory stack: the memory scopes their controller recalls \
                            through, person first. Absent, [person] with its defaults. The entry \
                            that wins for a seat is taken whole: an entry without scopes gets \
                            [person] even when memory.default lists more. Each id is spelled here; \
                            the cognition host judges the rest (see memory-scopes.md).",
            "type": "array",
            "items": {
                "type": "object",
                "additionalProperties": false,
                "required": ["id"],
                "properties": {
                    "id": {
                        "description": "The memory scope's id.",
                        "type": "string",
                        "pattern": policy,
                    },
                    "parameters": {
                        "description": "The scope's parameters, passed through as written: a \
                                        mapping with text keys and no YAML tags.",
                        "type": "object",
                    },
                },
            },
        },
    });
    if let (true, Some(map)) = (assignment, fields.as_object_mut()) {
        map.insert(
            "seat".to_owned(),
            json!({
                "description": "A person of the world listed under seats: in world.yaml.",
                "type": "string",
            }),
        );
        map.insert(
            "class".to_owned(),
            json!({
                "description": "A class of persons defined in configure/classes.yaml.",
                "type": "string",
            }),
        );
    }
    fields
}
