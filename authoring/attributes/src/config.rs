//! `configure/attributes.yaml` (`DECISIONS.md` `ARC-61`, `ARC-123`; `MODULE_SPEC.md` §4.1 "The world's
//! configuration"; `pr-s30d-sampler.md` §5.4).

use serde::Deserialize;

/// The world's choice of backend, mask, profile, population seed, salt and lock.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = serde_json::json!({
    "format": 1, "backend": "mineworld", "mask": "core", "profile": "san-diego-county",
    "seed": "fixture-pop-v1", "salt": 0, "lock": "cast"
}))]
pub struct AttributesConfiguration {
    /// The format version: 1.
    pub format: u32,
    /// The population profile people are sampled from, by its `profile:` id among the world's and its
    /// required packs' `populations/` files. Absent: no sampling.
    #[serde(default)]
    pub profile: Option<String>,
    /// The population seed: world content, never the run's `--seed`. Required when `profile` is set.
    #[serde(default)]
    pub seed: Option<String>,
    /// Changes sampled values only; the world's own population is the one this salt gives.
    #[serde(default)]
    pub salt: u32,
    /// The lock, by name: `locks/<name>.yaml` of this World Pack.
    #[serde(default)]
    pub lock: Option<String>,
    /// The person-generation backend, by id (`ARC-126`). Absent: the only backend the world's packs
    /// provide, or none.
    #[serde(default)]
    pub backend: Option<String>,
    /// The mask applied to the backend's catalogue, by id. Absent: the backend's default.
    #[serde(default)]
    pub mask: Option<String>,
}
