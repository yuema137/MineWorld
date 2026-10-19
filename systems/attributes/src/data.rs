//! The data this pack owns — `attributes/`, `vocabularies/`, `populations/` and `locks/` — and its world
//! configuration, built into one context: the frozen catalogue and the population. The running world
//! learns both through one context fact, `attributes-composed` (`ARC-121` item 5).

use std::any::Any;

use mineworld_attributes_model::backend::BackendFile;
use mineworld_attributes_model::config::AttributesConfiguration;
use mineworld_attributes_model::declaration::{AttributesFile, VocabulariesFile};
use mineworld_attributes_model::lock::LockFile;
use mineworld_attributes_model::mask::MaskFile;
use mineworld_attributes_model::profile::ProfileFile;
use mineworld_attributes_model::{
    AttributesComposed, AttributesContext, DataFile, facts::COMPOSED_SCHEMA,
};
use mineworld_authoring::{
    ConfigurationContext, DataDirectory, DataOrigin, DataRefusal, OwnsData, PackConfiguration,
    Seeding,
};
use mineworld_contracts::{Event, EventTypeId, Rejection, Visibility};
use mineworld_kernel::Emission;
use serde::Deserialize;
use serde::de::{Deserializer, Error as _};

use crate::system::AttributesSystem;

/// `attributes/`: declarations, overrides and `uses:`.
pub(crate) const ATTRIBUTES_DIRECTORY: DataDirectory = DataDirectory::from_static("attributes");

/// `vocabularies/`: value vocabularies.
pub(crate) const VOCABULARIES_DIRECTORY: DataDirectory = DataDirectory::from_static("vocabularies");

/// `populations/`: population profiles.
pub(crate) const POPULATIONS_DIRECTORY: DataDirectory = DataDirectory::from_static("populations");

/// `locks/`: a World Pack's locks (Q-S30d-2).
pub(crate) const LOCKS_DIRECTORY: DataDirectory = DataDirectory::from_static("locks");

/// `backends/`: a backend pack's contract, one file per pack (Q-S30e-1).
pub(crate) const BACKENDS_DIRECTORY: DataDirectory = DataDirectory::from_static("backends");

/// `masks/`: which of a backend's attributes exist in a world (`ARC-126`).
pub(crate) const MASKS_DIRECTORY: DataDirectory = DataDirectory::from_static("masks");

impl OwnsData for AttributesSystem {
    const DIRECTORIES: &'static [DataDirectory] = &[
        ATTRIBUTES_DIRECTORY,
        VOCABULARIES_DIRECTORY,
        POPULATIONS_DIRECTORY,
        LOCKS_DIRECTORY,
        BACKENDS_DIRECTORY,
        MASKS_DIRECTORY,
    ];
    type File = DataFile;
    type Context = AttributesContext;

    fn decode_file<'de, D: Deserializer<'de>>(
        directory: DataDirectory,
        deserializer: D,
    ) -> Result<DataFile, D::Error> {
        if directory == ATTRIBUTES_DIRECTORY {
            AttributesFile::deserialize(deserializer).map(DataFile::Attributes)
        } else if directory == VOCABULARIES_DIRECTORY {
            VocabulariesFile::deserialize(deserializer).map(DataFile::Vocabularies)
        } else if directory == POPULATIONS_DIRECTORY {
            ProfileFile::deserialize(deserializer).map(|file| DataFile::Profile(Box::new(file)))
        } else if directory == LOCKS_DIRECTORY {
            LockFile::deserialize(deserializer).map(|file| DataFile::Lock(Box::new(file)))
        } else if directory == BACKENDS_DIRECTORY {
            BackendFile::deserialize(deserializer).map(|file| DataFile::Backend(Box::new(file)))
        } else if directory == MASKS_DIRECTORY {
            MaskFile::deserialize(deserializer).map(|file| DataFile::Mask(Box::new(file)))
        } else {
            Err(D::Error::custom(format!(
                "attributes owns no directory '{directory}'"
            )))
        }
    }

    fn context(
        files: Vec<(DataOrigin, DataFile)>,
        configuration: Option<&dyn Any>,
    ) -> Result<AttributesContext, DataRefusal> {
        let configuration = configuration
            .and_then(|configuration| configuration.downcast_ref::<AttributesConfiguration>());
        AttributesContext::build(files, configuration).map_err(|refusal| DataRefusal {
            file: refusal.file,
            reason: refusal.reason,
        })
    }

    const CONTEXT_FACTS: &'static [EventTypeId] = &[AttributesComposed::EVENT_TYPE];

    /// One `attributes-composed`: the catalogue's run-time projection and the population's identity.
    fn seed_context(
        _: &Seeding<'_, '_>,
        context: &AttributesContext,
    ) -> Result<Vec<Emission>, Rejection> {
        let composed = AttributesComposed {
            schema: COMPOSED_SCHEMA,
            catalogue: context.catalogue.projection(),
            population: context
                .population
                .as_ref()
                .map(|population| population.identity()),
            composition: context
                .composition
                .as_ref()
                .map(|composed| composed.identity()),
        };
        Ok(vec![Emission::new::<AttributesComposed>(
            crate::system::encode(&composed),
            Visibility::SystemInternal,
        )])
    }
}

/// `configure/attributes.yaml` (`ARC-61`): profile, seed, salt, lock. It states no fact of its own; its
/// identity is carried by `attributes-composed` and compared at resume (`pr-s30d-sampler.md` §5.7).
impl PackConfiguration for AttributesSystem {
    type Configuration = AttributesConfiguration;
    const FACTS: &'static [EventTypeId] = &[];

    fn seed(
        _: &Seeding<'_, '_>,
        _: &AttributesConfiguration,
        _: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(Vec::new())
    }
}
