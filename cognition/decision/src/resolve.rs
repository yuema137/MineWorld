//! Resolving a decision list per seat, the operator's overrides, the digest and the header clause
//! (`docs/MODULE_SPEC.md` §4.1 rules 3–8; `pr-dl-a-decision-list.md` SD-DLa-6, SD-DLa-8, SD-DLa-11,
//! SD-DLa-14).
//!
//! ```text
//! model     --seat-model > seat entry > class entry > default: > rule; an external model followed
//!           down its fallback chain to the in-process model that drives the seat here, or to none
//! memory    seat entry > class entry > default; nothing when the file has no memory: section
//! ```
//!
//! Resolution is in roster order, and everything printed or hashed follows it.

use std::collections::BTreeSet;

use mineworld_contracts::EntityKey;
use serde::Serialize;

use crate::format::{ClassRef, Roster};
use crate::model::ModelId;
use crate::models::{Catalog, Decoded};
use crate::read::{DecisionList, Fallback, MemoryChoice};

/// An operator's `--seat-model SEAT=MODEL`, as given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatOverride {
    /// The seat.
    pub seat: EntityKey,
    /// The model id, as typed.
    pub model: String,
}

/// Where the list in effect came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// No file: no `decisions:` key and no `--decisions`.
    Implicit,
    /// The pack's `decisions:` file.
    Pack,
    /// The operator's `--decisions FILE`.
    File,
}

/// Where the list in effect came from, and whether `--seat-model` changed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Source {
    /// The file's origin.
    pub origin: Origin,
    /// Whether any `--seat-model` was given.
    pub overridden: bool,
}

impl Source {
    /// Whether a decision list is in effect: a file or an override. With none, every output is as it
    /// was before decision lists existed (`docs/MODULE_SPEC.md` §4.1 rule 6).
    pub fn in_effect(self) -> bool {
        self.origin != Origin::Implicit || self.overridden
    }
}

/// Where a memory choice was written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceSpan {
    /// The file as written in `world.yaml` or passed to `--decisions`.
    pub file: String,
    /// The winning entry's line.
    pub line: u64,
    /// Its column.
    pub column: u64,
}

/// One seat's memory choice, resolved: FG-b's `MemoryPolicyChoice`, passed through (QFGb-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryPolicyChoice {
    /// The choice as written.
    pub choice: MemoryChoice,
    /// Where it was written.
    pub source: SourceSpan,
}

/// One seat, resolved.
#[derive(Debug, Clone)]
pub struct ResolvedSeat {
    /// The seat.
    pub seat: EntityKey,
    /// The in-process model that drives the seat in a host that runs no out-of-process model; `None`
    /// when its chain ends at `none`.
    pub drives: Option<Decoded>,
    /// The external model the list assigned, when it did; then `chain` is the way to `drives`.
    pub assigned: Option<ModelId>,
    /// The ids followed from `assigned`, ending at `drives`' id or `none`; empty when no external
    /// model was assigned.
    pub chain: Vec<String>,
    /// The memory choice; `None` when no `memory:` entry and no `memory:` default applies.
    pub memory: Option<MemoryPolicyChoice>,
}

/// A decision list resolved for every seat, in roster order.
#[derive(Debug, Clone)]
pub struct ResolvedDecisions {
    source: Source,
    seats: Vec<ResolvedSeat>,
    memory_present: bool,
}

impl<C: Catalog> DecisionList<C> {
    /// Resolves every seat of `roster`, after applying `overrides` (`--seat-model`).
    ///
    /// # Errors
    ///
    /// An override naming a seat the world does not offer, a model this build does not install, an
    /// external model without a fallback in this list, or one seat twice — naming the flag.
    pub fn resolve(
        &self,
        roster: &Roster,
        origin: Origin,
        overrides: &[SeatOverride],
    ) -> Result<ResolvedDecisions, String> {
        let overridden = self.overrides(roster, overrides)?;
        let mut seats = Vec::with_capacity(roster.seats.len());
        for entry in &roster.seats {
            let id = overridden
                .iter()
                .find(|(seat, _)| *seat == entry.seat)
                .map(|(_, id)| id)
                .or_else(|| self.seats.get(&entry.seat))
                .or_else(|| self.classes.get(&entry.class))
                .or(self.default.as_ref())
                .map_or("rule", ModelId::as_str);
            let (drives, assigned, chain) = self.follow(id);
            seats.push(ResolvedSeat {
                seat: entry.seat.clone(),
                drives,
                assigned,
                chain,
                memory: self.memory_of(&entry.seat, &entry.class),
            });
        }
        Ok(ResolvedDecisions {
            source: Source {
                origin,
                overridden: !overrides.is_empty(),
            },
            seats,
            memory_present: self.memory.is_some(),
        })
    }

    fn overrides(
        &self,
        roster: &Roster,
        overrides: &[SeatOverride],
    ) -> Result<Vec<(EntityKey, ModelId)>, String> {
        let mut checked: Vec<(EntityKey, ModelId)> = Vec::new();
        for given in overrides {
            let flag = format!("--seat-model {}={}", given.seat, given.model);
            if roster.seat(&given.seat).is_none() {
                let offered: BTreeSet<&EntityKey> =
                    roster.seats.iter().map(|entry| &entry.seat).collect();
                return Err(format!(
                    "{flag}: this world offers no such seat. It offers: {}",
                    offered
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            if checked.iter().any(|(seat, _)| *seat == given.seat) {
                return Err(format!("{flag}: seat '{}' is named twice", given.seat));
            }
            let model = C::find(&given.model).ok_or_else(|| {
                format!(
                    "{flag}: '{}' is not a model this build installs; it installs: {}",
                    given.model,
                    C::listed()
                )
            })?;
            let id = ModelId::new(model.id()).map_err(|error| format!("{flag}: {error}"))?;
            if model.is_external() && !self.models.contains_key(&id) {
                return Err(format!(
                    "{flag}: '{id}' runs out of process and has no fallback; name it under models: in \
                     the decision list (--decisions) with one"
                ));
            }
            checked.push((given.seat.clone(), id));
        }
        Ok(checked)
    }

    /// The model `id` resolves to here: itself when in process, else the end of its chain.
    fn follow(&self, id: &str) -> (Option<Decoded>, Option<ModelId>, Vec<String>) {
        let parameters = |id: &str| -> Option<Decoded> {
            let model = C::find(id)?;
            ModelId::new(id)
                .ok()
                .and_then(|key| self.models.get(&key))
                .and_then(|listed| listed.parameters.clone())
                .or_else(|| model.defaults())
        };
        let external = |id: &str| C::find(id).is_some_and(Catalog::is_external);
        if !external(id) {
            return (parameters(id), None, Vec::new());
        }
        let assigned = ModelId::new(id).ok();
        let mut chain = vec![id.to_owned()];
        let mut current = id.to_owned();
        // Checked at load: at most CHAIN_LINKS links, no cycle, every external link listed.
        loop {
            let next = ModelId::new(current.as_str())
                .ok()
                .and_then(|key| self.models.get(&key))
                .and_then(|listed| listed.fallback.clone());
            match next {
                Some(Fallback::Model(next)) => {
                    chain.push(next.as_str().to_owned());
                    if !external(next.as_str()) {
                        return (parameters(next.as_str()), assigned, chain);
                    }
                    current = next.as_str().to_owned();
                }
                Some(Fallback::None) | None => {
                    chain.push("none".to_owned());
                    return (None, assigned, chain);
                }
            }
        }
    }

    fn memory_of(&self, seat: &EntityKey, class: &ClassRef) -> Option<MemoryPolicyChoice> {
        let memory = self.memory.as_ref()?;
        let choice = memory
            .seats
            .get(seat)
            .or_else(|| memory.classes.get(class))
            .or(memory.default.as_ref())?;
        Some(MemoryPolicyChoice {
            choice: choice.clone(),
            source: SourceSpan {
                file: self.file.clone(),
                line: choice.at.line,
                column: choice.at.column,
            },
        })
    }
}

impl ResolvedDecisions {
    /// Where the list came from.
    pub fn source(&self) -> Source {
        self.source
    }

    /// Every seat, in roster order.
    pub fn seats(&self) -> &[ResolvedSeat] {
        &self.seats
    }

    /// Whether the file has a `memory:` section.
    pub fn memory_present(&self) -> bool {
        self.memory_present
    }

    /// The model driving the seat at roster index `k`, or `None` for a seat whose chain ends at `none`.
    pub fn drives(&self, k: usize) -> Option<&Decoded> {
        self.seats.get(k).and_then(|seat| seat.drives.as_ref())
    }

    /// FNV-1a 64 over the resolved list's canonical encoding: each seat's key, effective model,
    /// canonical parameters, chain, and — only when the file has `memory:` — its memory choice without
    /// its source position. For a person to compare two invocations by eye; not evidence.
    pub fn digest(&self) -> u64 {
        let canonical: Vec<Canonical<'_>> = self
            .seats
            .iter()
            .map(|seat| Canonical {
                seat: seat.seat.as_str(),
                model: seat.drives.as_ref().map(|drives| drives.id().as_str()),
                parameters: seat.drives.as_ref().map(Decoded::canonical),
                chain: &seat.chain,
                memory: self.memory_present.then(|| {
                    seat.memory.as_ref().map(|memory| CanonicalMemory {
                        policy: memory.choice.policy.as_str(),
                        fallback: memory.choice.fallback.as_ref().map(|id| id.as_str()),
                        parameters: &memory.choice.parameters,
                    })
                }),
            })
            .collect();
        let bytes = serde_json::to_vec(&canonical).expect("the canonical form is JSON");
        fnv1a(&bytes)
    }

    /// The clause `run` appends to its header, and `server --town` to its `town:` statement, when a
    /// list is in effect: `; decisions <digest> (<n> <model>, …)`, then one `; <seat>: <assigned> → …
    /// (<why>)` per seat assigned an external model. `None` when no list is in effect.
    pub fn clause(&self, why: &str) -> Option<String> {
        if !self.source.in_effect() {
            return None;
        }
        let mut counts: Vec<(&str, usize)> = Vec::new();
        for seat in &self.seats {
            let name = seat
                .drives
                .as_ref()
                .map_or("idle", |drives| drives.id().as_str());
            match counts.iter_mut().find(|(model, _)| *model == name) {
                Some((_, count)) => *count += 1,
                None => counts.push((name, 1)),
            }
        }
        let counted: Vec<String> = counts
            .iter()
            .map(|(model, count)| format!("{count} {model}"))
            .collect();
        let mut clause = format!(
            "; decisions {:016x} ({})",
            self.digest(),
            counted.join(", ")
        );
        for seat in self.seats.iter().filter(|seat| seat.assigned.is_some()) {
            clause.push_str(&format!(
                "; {}: {} ({why})",
                seat.seat,
                seat.chain.join(" → ")
            ));
        }
        Some(clause)
    }
}

#[derive(Serialize)]
struct Canonical<'a> {
    seat: &'a str,
    model: Option<&'a str>,
    parameters: Option<&'a str>,
    chain: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    memory: Option<Option<CanonicalMemory<'a>>>,
}

#[derive(Serialize)]
struct CanonicalMemory<'a> {
    policy: &'a str,
    fallback: Option<&'a str>,
    parameters: &'a serde_json::Map<String, serde_json::Value>,
}

/// FNV-1a 64, as `run`'s history fingerprint computes it.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
