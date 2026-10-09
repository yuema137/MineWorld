//! The World's Interaction List: one section shape for every pack, typed and enforced by its owner
//! (`docs/DECISIONS.md` `ARC-63`, `ARC-64`, `ARC-65`, `DEP-28`; `docs/MODULE_SPEC.md` §4.2).
//!
//! ```text
//! decl        what a pack declares its section may say: roles, actions, facts, effects, audiences
//! selector    `*` or a class, one per role; specificity, overlap, matching
//! section     the authored section and its decoding (refusals at their line and column); parameters!
//! resolve     levels, then specificity, then forbid; ambiguity refused at load; one sorted value
//! lookup      the configured fact, the per-Place component, and permits / parameters / consequence
//! biography   the selection the biography projection asks
//! ```
//!
//! The SDK supplies the shape; the pack supplies the meaning. Nothing here names a pack.

pub mod biography;
mod decl;
mod lookup;
mod resolve;
mod section;

use std::sync::Arc;

use mineworld_authoring::{
    AuthoredConfiguration, ConfigurationContext, ConfigurationRefusal, Reference, Seeding,
};
use mineworld_contracts::{
    ComponentTypeId, EntityType, EventTypeId, Rejection, RejectionCode, SystemId, Visibility,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Deserializer};

pub use biography::{Configured, ConsequenceTable, Known, consequences, selected};
pub use decl::{ActionDecl, Audience, Effect, FactDecl, Position, Role, SectionDecl};
pub use lookup::{
    Consequence, Interactions, SectionConfigured, consequence, declare, install, parameters,
    permits, reduce,
};
pub use resolve::{Chosen, Resolution, Resolved, resolve};
pub use section::{
    ConsEntry, EXTENDS_MAX, Entries, Fields, ParamEntry, Parameters, Partial, RuleEntry, Section,
};
pub use selector::{RoleSubjects, Roles, Selector, Selectors};

mod selector;

/// Implemented by a System Pack that has a section of the World's Interaction List (`ARC-63` item 2).
/// The SDK decodes, resolves and looks up; the pack declares what its section may say, enforces the
/// answers, and writes `mineworld_sdk::interactions!();` in its `impl SystemPack`.
///
/// A pack is a unit struct with no state (`INV-7`), so deriving `Debug`, `Clone` and `PartialEq` on it
/// costs nothing; the section's types are generic over it.
pub trait InteractionSection:
    crate::SystemPack + Clone + core::fmt::Debug + PartialEq + Send + Sync + 'static
{
    /// The pack's typed parameter block, made by [`parameters!`](crate::parameters).
    type Parameters: Parameters;

    /// Pack-specific consequence knobs; `()` for none.
    type Knobs: Fields;

    /// Its configured fact: `<pack id>-interactions-configured`.
    const CONFIGURED: EventTypeId;

    /// Its per-Place component: `<pack id>-interactions`.
    const COMPONENT: ComponentTypeId;

    /// Each action a rule may name.
    const ACTIONS: &'static [ActionDecl] = &[];

    /// Each of its facts a consequence may name.
    const FACTS: &'static [FactDecl] = &[];

    /// The roles a parameter entry may scope by.
    const PARAMETER_ROLES: &'static [Role] = &[];

    /// Its compiled reference lists (`ARC-63` item 4): always `default`, today's behaviour.
    fn reference_lists() -> Vec<(&'static str, Section<Self>)> {
        vec![("default", Section::empty())]
    }

    /// Its configured fact's payload, with the pack's own codec.
    fn encode(resolved: &Resolved<Self>) -> Vec<u8>;

    /// The payload read back.
    ///
    /// # Errors
    ///
    /// The codec's refusal, worded.
    fn decode(payload: &[u8]) -> Result<Resolved<Self>, String>;
}

impl SectionDecl {
    /// What the installed set says about `S`'s section. Used by [`interactions!`](crate::interactions).
    pub const fn of<S: InteractionSection>() -> Self {
        Self {
            configured: S::CONFIGURED,
            actions: S::ACTIONS,
            facts: S::FACTS,
            biographical: <S as crate::SystemPack>::BIOGRAPHICAL,
            consequences: consequences::<S>,
        }
    }
}

/// A section, held by the loader as a configuration (`ARC-61`): what `interactions!()` decodes into.
#[derive(Debug)]
struct HeldSection<S: InteractionSection>(Section<S>);

/// The code a seed refusal carries; unreachable after the loader's own check.
const REFUSED: RejectionCode = RejectionCode::from_static("interaction-section-refused");

impl<S: InteractionSection> AuthoredConfiguration for HeldSection<S> {
    fn owner(&self) -> SystemId {
        <S as SystemIdentity>::ID
    }

    fn references(&self) -> Vec<Reference<'_>> {
        self.0
            .regions
            .keys()
            .map(|key| Reference {
                key,
                entity_type: EntityType::Place,
            })
            .collect()
    }

    fn requires(&self) -> Vec<SystemId> {
        Vec::new()
    }

    fn facts(&self) -> &'static [EventTypeId] {
        <S as crate::SystemPack>::CONFIGURATION_FACTS
    }

    fn attachments(&self) -> Vec<&mineworld_authoring::Attachment> {
        Vec::new()
    }

    fn check(&self, context: &ConfigurationContext<'_>) -> Result<(), ConfigurationRefusal> {
        resolve(&self.0, context.classes()).map(|_| ())
    }

    fn seed(
        &self,
        _: &Seeding<'_, '_>,
        context: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        let resolved =
            resolve(&self.0, context.classes()).map_err(|refusal| Rejection::System {
                code: REFUSED,
                detail: Some(format!("{refusal:?}")),
            })?;
        Ok(vec![Emission::new::<SectionConfigured<S>>(
            S::encode(&resolved),
            Visibility::SystemInternal,
        )])
    }
}

/// Decodes `configure/<pack>.yaml` as `S`'s section, straight from the stream. Used by
/// [`interactions!`](crate::interactions).
///
/// # Errors
///
/// The decoder's error, at its line and column.
pub fn decode<'de, S: InteractionSection, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Arc<dyn AuthoredConfiguration>, D::Error> {
    let section = Section::<S>::deserialize(deserializer)?;
    Ok(Arc::new(HeldSection(section)))
}

/// Declares, inside an `impl SystemPack`, that the pack's configuration is its section of the World's
/// Interaction List (`ARC-63`): defines `CONFIGURATION`, `CONFIGURATION_FACTS`, `INTERACTIONS` and
/// `decode_configuration` from its [`InteractionSection`] impl, so a pack with a section has no other
/// configuration.
///
/// ```text
/// impl SystemPack for ExampleSystem {
///     const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
///     mineworld_sdk::interactions!();
/// }
/// ```
#[macro_export]
macro_rules! interactions {
    () => {
        const CONFIGURATION: ::core::option::Option<$crate::__private::SystemId> =
            ::core::option::Option::Some(<Self as $crate::__private::SystemIdentity>::ID);

        const CONFIGURATION_FACTS: &'static [$crate::__private::EventTypeId] =
            &[<Self as $crate::interactions::InteractionSection>::CONFIGURED];

        const INTERACTIONS: ::core::option::Option<$crate::interactions::SectionDecl> =
            ::core::option::Option::Some($crate::interactions::SectionDecl::of::<Self>());

        fn decode_configuration<'de, D: $crate::__private::Deserializer<'de>>(
            deserializer: D,
        ) -> ::core::result::Result<
            $crate::__private::Arc<dyn $crate::__private::AuthoredConfiguration>,
            D::Error,
        > {
            $crate::interactions::decode::<Self, D>(deserializer)
        }
    };
}

#[cfg(test)]
mod tests;
