//! The data this pack owns: `attributes/` and `vocabularies/`, built into the world's catalogue, which
//! the running world learns through one context fact, `attributes-composed` (`ARC-121` item 5).

use std::any::Any;

use mineworld_attributes_model::declaration::{AttributesFile, VocabulariesFile};
use mineworld_attributes_model::{AttributesComposed, Catalogue, DataFile, facts::COMPOSED_SCHEMA};
use mineworld_authoring::{DataDirectory, DataOrigin, DataRefusal, OwnsData, Seeding};
use mineworld_contracts::{Event, EventTypeId, Rejection, Visibility};
use mineworld_kernel::Emission;
use serde::Deserialize;
use serde::de::{Deserializer, Error as _};

use crate::system::AttributesSystem;

/// `attributes/`: declarations, overrides and `uses:`.
pub(crate) const ATTRIBUTES_DIRECTORY: DataDirectory = DataDirectory::from_static("attributes");

/// `vocabularies/`: value vocabularies.
pub(crate) const VOCABULARIES_DIRECTORY: DataDirectory = DataDirectory::from_static("vocabularies");

impl OwnsData for AttributesSystem {
    const DIRECTORIES: &'static [DataDirectory] = &[ATTRIBUTES_DIRECTORY, VOCABULARIES_DIRECTORY];
    type File = DataFile;
    type Context = Catalogue;

    fn decode_file<'de, D: Deserializer<'de>>(
        directory: DataDirectory,
        deserializer: D,
    ) -> Result<DataFile, D::Error> {
        if directory == ATTRIBUTES_DIRECTORY {
            AttributesFile::deserialize(deserializer).map(DataFile::Attributes)
        } else if directory == VOCABULARIES_DIRECTORY {
            VocabulariesFile::deserialize(deserializer).map(DataFile::Vocabularies)
        } else {
            Err(D::Error::custom(format!(
                "attributes owns no directory '{directory}'"
            )))
        }
    }

    fn context(
        files: Vec<(DataOrigin, DataFile)>,
        _: Option<&dyn Any>,
    ) -> Result<Catalogue, DataRefusal> {
        Catalogue::build(&files).map_err(|refusal| DataRefusal {
            file: refusal.file().map(std::path::Path::to_path_buf),
            reason: refusal.to_string(),
        })
    }

    const CONTEXT_FACTS: &'static [EventTypeId] = &[AttributesComposed::EVENT_TYPE];

    /// One `attributes-composed`, holding the catalogue's run-time projection, about nobody.
    fn seed_context(
        _: &Seeding<'_, '_>,
        catalogue: &Catalogue,
    ) -> Result<Vec<Emission>, Rejection> {
        let composed = AttributesComposed {
            schema: COMPOSED_SCHEMA,
            catalogue: catalogue.projection(),
        };
        Ok(vec![Emission::new::<AttributesComposed>(
            crate::system::encode(&composed),
            Visibility::SystemInternal,
        )])
    }
}
