//! `attributes`' data context (`DECISIONS.md` `ARC-121`, `ARC-123`): the frozen catalogue and, when the
//! world's configuration names one, its population — the profile, the lock, the seed and the salt.

use std::path::PathBuf;

use mineworld_authoring::{DataOrigin, DataSource};
use serde::Serialize;

use crate::catalogue::{AttributeError, Catalogue};
use crate::config::AttributesConfiguration;
use crate::declaration::DataFile;
use crate::lock::Lock;
use crate::profile::{Profile, ProfileError, build};

/// The world's population: what genesis samples from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Population {
    /// The profile, when the configuration names one.
    pub profile: Option<Profile>,
    /// The population seed (empty without a profile).
    pub seed: String,
    /// The salt.
    pub salt: u32,
    /// The lock's name, when the configuration names one.
    pub lock_name: Option<String>,
    /// The lock, when the configuration names one and the world holds it.
    pub lock: Option<Lock>,
}

/// The population's identity in `attributes-composed` (`pr-s30d-sampler.md` §5.7): a resumed world whose
/// profile, seed, salt or lock changed is drift.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PopulationIdentity {
    /// The profile's id, version and digest (FNV-1a 64 of its canonical JSON, in hex).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<(String, u32, String)>,
    /// The seed.
    pub seed: String,
    /// The salt.
    pub salt: u32,
    /// FNV-1a 64 of the lock's canonical form, in hex.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lock_digest: Option<String>,
}

impl Population {
    /// Its identity.
    pub fn identity(&self) -> PopulationIdentity {
        PopulationIdentity {
            profile: self.profile.as_ref().map(|profile| {
                (
                    profile.id.clone(),
                    profile.version,
                    format!("{:016x}", profile.digest()),
                )
            }),
            seed: self.seed.clone(),
            salt: self.salt,
            lock_digest: self
                .lock
                .as_ref()
                .map(|lock| format!("{:016x}", lock.digest())),
        }
    }
}

/// `attributes`' context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributesContext {
    /// The frozen catalogue.
    pub catalogue: Catalogue,
    /// The population, when the world configures `attributes`.
    pub population: Option<Population>,
}

/// Why the context was refused: the file it is about, and the refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRefusal {
    /// The file, when there is one.
    pub file: Option<PathBuf>,
    /// The refusal, named.
    pub reason: String,
}

impl From<AttributeError> for ContextRefusal {
    fn from(error: AttributeError) -> Self {
        Self {
            file: error.file().map(std::path::Path::to_path_buf),
            reason: error.to_string(),
        }
    }
}

impl From<ProfileError> for ContextRefusal {
    fn from(error: ProfileError) -> Self {
        Self {
            file: None,
            reason: error.to_string(),
        }
    }
}

impl AttributesContext {
    /// The context from every data file in collection order and the world's configuration.
    ///
    /// # Errors
    ///
    /// Every refusal of the catalogue, the profile and the lock, by name.
    pub fn build(
        files: Vec<(DataOrigin, DataFile)>,
        configuration: Option<&AttributesConfiguration>,
    ) -> Result<Self, ContextRefusal> {
        let mut declarations = Vec::new();
        let mut profiles = Vec::new();
        let mut locks = Vec::new();
        for (origin, file) in files {
            match file {
                DataFile::Profile(profile) => profiles.push((origin.path, *profile)),
                DataFile::Lock(lock) => {
                    if origin.source != DataSource::World {
                        return Err(ContextRefusal {
                            file: Some(origin.path),
                            reason: "BadDeclaration: a lock belongs to one world; only a World Pack holds locks/".to_owned(),
                        });
                    }
                    locks.push((origin.path, *lock));
                }
                other => declarations.push((origin, other)),
            }
        }
        let catalogue = Catalogue::build(&declarations)?;
        let Some(configuration) = configuration else {
            return Ok(Self {
                catalogue,
                population: None,
            });
        };
        if configuration.format != 1 {
            return Err(ContextRefusal {
                file: None,
                reason: format!(
                    "configure/attributes.yaml: format {} is not 1",
                    configuration.format
                ),
            });
        }
        let profile = build(&catalogue, &profiles, Some(configuration))?;
        let lock = match &configuration.lock {
            None => None,
            Some(name) => {
                let wanted = format!("{name}.yaml");
                let found = locks.iter().find(|(path, _)| {
                    path.file_name()
                        .is_some_and(|file| file.to_string_lossy() == wanted)
                });
                match found {
                    Some((path, file)) => Some(Lock::check(file, &catalogue).map_err(
                        |reason| ContextRefusal {
                            file: Some(path.clone()),
                            reason,
                        },
                    )?),
                    None => None,
                }
            }
        };
        Ok(Self {
            catalogue,
            population: Some(Population {
                profile,
                seed: configuration.seed.clone().unwrap_or_default(),
                salt: configuration.salt,
                lock_name: configuration.lock.clone(),
                lock,
            }),
        })
    }
}
