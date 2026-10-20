//! `attributes`' data context (`DECISIONS.md` `ARC-121`, `ARC-123`): the frozen catalogue and, when the
//! world's configuration names one, its population — the profile, the lock, the seed and the salt.

use std::collections::BTreeSet;
use std::path::PathBuf;

use mineworld_authoring::{DataOrigin, DataSource};
use serde::Serialize;

use crate::backend::{Backend, MINEWORLD, choose};
use crate::catalogue::{AttributeError, CORE_PACK, Catalogue, Composition};
use crate::config::AttributesConfiguration;
use crate::declaration::DataFile;
use crate::lock::Lock;
use crate::mask::Mask;
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
    /// The backend and mask the world uses, when its packs provide a backend (S30-e).
    pub composition: Option<Composed>,
    /// Every backend the world's packs provide.
    pub backends: Vec<Backend>,
}

/// The backend and mask a world's catalogue was composed under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composed {
    /// The backend.
    pub backend: Backend,
    /// The mask.
    pub mask: Mask,
    /// The backend's default mask, which carries the `core` reader lines (§5.6).
    pub default_mask: Option<Mask>,
}

/// The composition's identity in `attributes-composed`: a resumed world whose backend or mask changed is
/// drift (`pr-s30e-builtin-catalogue.md` §5.3 item 5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionIdentity {
    /// The backend's id and version.
    pub backend: (String, String),
    /// The mask's id and version.
    pub mask: (String, u32),
}

impl Composed {
    /// Its identity.
    pub fn identity(&self) -> CompositionIdentity {
        CompositionIdentity {
            backend: (self.backend.id.clone(), self.backend.version.clone()),
            mask: (self.mask.id.clone(), self.mask.version),
        }
    }
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
        let mut backends = Vec::new();
        let mut masks = Vec::new();
        for (origin, file) in files {
            match file {
                DataFile::Profile(profile) => profiles.push((origin.path, *profile)),
                DataFile::Backend(backend) => {
                    let checked = Backend::check(&origin.pack, &backend).map_err(|reason| {
                        ContextRefusal {
                            file: Some(origin.path.clone()),
                            reason,
                        }
                    })?;
                    backends.push((origin.path, checked));
                }
                DataFile::Mask(mask) => {
                    let checked = Mask::check(&mask).map_err(|reason| ContextRefusal {
                        file: Some(origin.path.clone()),
                        reason,
                    })?;
                    masks.push((origin, checked));
                }
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
        let chosen = choose(&backends, configuration.and_then(|c| c.backend.as_deref()))
            .map_err(|reason| ContextRefusal { file: None, reason })?
            .cloned();
        let composition = match chosen {
            None => {
                if let Some(mask) = configuration.and_then(|c| c.mask.as_ref()) {
                    return Err(ContextRefusal {
                        file: None,
                        reason: format!(
                            "UnknownMask: '{mask}' is named, but the world's packs provide no backend"
                        ),
                    });
                }
                None
            }
            Some(backend) => {
                let wanted = configuration
                    .and_then(|c| c.mask.clone())
                    .unwrap_or_else(|| backend.masks.default.clone());
                let mask = if wanted == crate::mask::ALL {
                    Mask::all()
                } else {
                    masks
                        .iter()
                        .find(|(origin, mask)| {
                            mask.id == wanted && (origin.pack == backend.pack || origin.source == DataSource::World)
                        })
                        .map(|(_, mask)| mask.clone())
                        .ok_or_else(|| ContextRefusal {
                            file: None,
                            reason: format!(
                                "UnknownMask: neither the backend '{}' nor the world holds the mask '{wanted}'",
                                backend.id
                            ),
                        })?
                };
                if backend.id != MINEWORLD
                    && !declarations
                        .iter()
                        .any(|(origin, _)| origin.pack == CORE_PACK)
                {
                    return Err(ContextRefusal {
                        file: None,
                        reason: format!(
                            "BackendMalformed: the backend of '{}', field 'catalogues': the appearance group comes from {CORE_PACK}, which the world does not require",
                            backend.pack
                        ),
                    });
                }
                let default_mask = masks
                    .iter()
                    .find(|(origin, mask)| {
                        origin.pack == backend.pack && mask.id == backend.masks.default
                    })
                    .map(|(_, mask)| mask.clone());
                Some(Composed {
                    backend,
                    mask,
                    default_mask,
                })
            }
        };
        let backend_packs: BTreeSet<String> = backends
            .iter()
            .map(|(_, backend)| backend.pack.clone())
            .collect();
        let catalogue = Catalogue::build_composed(
            &declarations,
            composition
                .as_ref()
                .map(|composed| Composition {
                    backend: &composed.backend,
                    mask: &composed.mask,
                    backend_packs: &backend_packs,
                })
                .as_ref(),
        )?;
        let provided = backends.into_iter().map(|(_, backend)| backend).collect();
        let Some(configuration) = configuration else {
            return Ok(Self {
                catalogue,
                population: None,
                composition,
                backends: provided,
            });
        };
        if configuration.profile.is_some()
            && let Some(composed) = &composition
            && !composed.backend.sampling.contains("profile")
        {
            return Err(ContextRefusal {
                file: None,
                reason: format!(
                    "UnsupportedSampling: the backend '{}' does not sample from a profile",
                    composed.backend.id
                ),
            });
        }
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
            composition,
            backends: provided,
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
