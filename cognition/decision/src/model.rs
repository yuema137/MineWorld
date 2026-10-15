//! The trait every in-process decision model implements, and the object-safe form a host holds.
//!
//! ```text
//! DecisionModel        a model type: an id, a package, typed parameters, decide and step — pure, &self
//! Configured<M>        that model with the parameters a decision list gave it
//! SeatModel            what a host holds per seat: Arc<dyn SeatModel>, built by the installed set
//! ```
//!
//! `DecisionModel` has a constant and an associated type, so it is not object-safe; a host never names a
//! model type, only `SeatModel` (`pr-dl-a-decision-list.md` SD-DLa-1).

use core::fmt;

use mineworld_contracts::{ActionRequest, Observation, SimDuration, SystemId};
use mineworld_sdk::Package;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::parameters::ModelParameters;

/// The run's seed, the same for every seat. Every draw a model takes is mixed from it ([`crate::Draw`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Seed(u64);

impl Seed {
    /// A seed.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Its value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// What every consult is given besides the model's parameters: host context, never preferences.
#[derive(Debug, Clone, Copy)]
pub struct Consult<'a> {
    /// The run's seed.
    pub seed: Seed,
    /// The answer window: the host's consult period, in world seconds. A line heard in
    /// `(t − pace, t]` is the consult at `t`'s to answer.
    pub pace: SimDuration,
    /// What the seat perceives now, and nothing else (`INV-13`).
    pub observation: &'a Observation<Value>,
}

/// A decision model's id, as `decisions.yaml` names it: the identifier rule of a system id (lowercase
/// ASCII letters, digits, `-` and `_`, 1 to 64 bytes, no separator at either end).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ModelId(SystemId);

impl ModelId {
    /// Validates an id.
    ///
    /// # Errors
    ///
    /// The identifier rule's refusal.
    pub fn new(value: impl Into<String>) -> Result<Self, mineworld_contracts::ContractError> {
        SystemId::new(value).map(Self)
    }

    /// The id as text.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<'de> Deserialize<'de> for ModelId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::new(text.as_str()).map_err(|error| {
            serde::de::Error::custom(format!("'{text}' is not a model id: {error}"))
        })
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An in-process decision model: what a seat attempts, as a pure function of what it is given.
///
/// `&self`, no interior mutability (each model crate's `clippy.toml`), no clock, and no randomness but
/// [`crate::Draw`] over [`Consult::seed`]: what a model returns is a function of `(parameters, seed,
/// pace, observation)`, so a model rebuilt after a restart decides exactly as the one that died
/// (`DECISIONS.md` `ARC-27`). A model returns at most one request, which the world revalidates at
/// dispatch; its parameters change preferences among what the world offers, never what is offered.
pub trait DecisionModel: Default + Send + Sync + 'static {
    /// The id `decisions.yaml` names, such as `rule`. The installed set's guard test checks it against
    /// the identifier rule and refuses the reserved `none`.
    const ID: &'static str;
    /// The model crate's package identity (`DECISIONS.md` `ARC-53`): `mineworld_sdk::package!()`.
    const PACKAGE: Package;
    /// Typed, bounded parameters with defaults; `()` for none.
    type Parameters: ModelParameters;

    /// The consult: at most one request.
    fn decide(&self, parameters: &Self::Parameters, consult: &Consult<'_>)
    -> Option<ActionRequest>;

    /// The next stride of a disclosed walk, between consults; nothing when there is no walk to step.
    fn step(&self, parameters: &Self::Parameters, consult: &Consult<'_>) -> Option<ActionRequest>;
}

/// The object-safe form a host holds per seat: a model with its decoded parameters.
pub trait SeatModel: Send + Sync {
    /// The model's id.
    fn id(&self) -> &ModelId;

    /// [`DecisionModel::decide`] with this seat's parameters.
    fn decide(&self, consult: &Consult<'_>) -> Option<ActionRequest>;

    /// [`DecisionModel::step`] with this seat's parameters.
    fn step(&self, consult: &Consult<'_>) -> Option<ActionRequest>;
}

/// `SeatModel` stays object-safe: a host holds `Arc<dyn SeatModel>`, and this line stops compiling first.
const _: Option<&dyn SeatModel> = None;

/// A model with the parameters a decision list gave it.
#[derive(Debug, Clone)]
pub struct Configured<M: DecisionModel> {
    model: M,
    parameters: M::Parameters,
    id: ModelId,
}

impl<M: DecisionModel> Configured<M> {
    /// `M` with `parameters`.
    ///
    /// # Panics
    ///
    /// When `M::ID` breaks the identifier rule — which the installed set's guard test refuses before any
    /// build ships.
    pub fn new(parameters: M::Parameters) -> Self {
        Self {
            model: M::default(),
            parameters,
            id: ModelId::new(M::ID).unwrap_or_else(|error| panic!("model id '{}': {error}", M::ID)),
        }
    }

    /// The parameters.
    pub const fn parameters(&self) -> &M::Parameters {
        &self.parameters
    }
}

impl<M: DecisionModel> Default for Configured<M> {
    fn default() -> Self {
        Self::new(M::Parameters::default())
    }
}

impl<M: DecisionModel> SeatModel for Configured<M> {
    fn id(&self) -> &ModelId {
        &self.id
    }

    fn decide(&self, consult: &Consult<'_>) -> Option<ActionRequest> {
        self.model.decide(&self.parameters, consult)
    }

    fn step(&self, consult: &Consult<'_>) -> Option<ActionRequest> {
        self.model.step(&self.parameters, consult)
    }
}
