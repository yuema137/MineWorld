//! Which licences a pack may carry (`DECISIONS.md` `ARC-55`).

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::error::PackageError;
use crate::identity::License;

/// The licences a world's packs may carry: a set of SPDX licence identifiers. A typed value, so a
/// world will be able to narrow or extend it through the generic configuration seam (`ARC-61`) —
/// `configure/packages.yaml` decodes into this type — rather than through a key of its own.
///
/// ```yaml
/// allowed: [MIT, Apache-2.0]
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Stated")]
pub struct LicencePolicy {
    allowed: BTreeSet<&'static str>,
}

/// The policy as a file states it, before its identifiers are checked.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Stated {
    allowed: Vec<String>,
}

/// The default: the common permissive set compatible with MIT redistribution (`ARC-55` point 1).
const DEFAULT: [&str; 8] = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
    "CC0-1.0",
    "Unlicense",
];

impl Default for LicencePolicy {
    fn default() -> Self {
        Self::new(DEFAULT).expect("the default lists SPDX identifiers")
    }
}

impl TryFrom<Stated> for LicencePolicy {
    type Error = PackageError;

    fn try_from(stated: Stated) -> Result<Self, Self::Error> {
        Self::new(stated.allowed.iter().map(String::as_str))
    }
}

impl LicencePolicy {
    /// A policy allowing exactly `identifiers`.
    ///
    /// # Errors
    ///
    /// [`PackageError::License`] for an identifier that is not on the SPDX licence list.
    pub fn new<'a>(identifiers: impl IntoIterator<Item = &'a str>) -> Result<Self, PackageError> {
        let mut allowed = BTreeSet::new();
        for identifier in identifiers {
            let known = spdx::license_id(identifier).ok_or_else(|| PackageError::License {
                text: identifier.to_owned(),
                reason: "not an SPDX licence identifier".to_owned(),
            })?;
            allowed.insert(known.name);
        }
        Ok(Self { allowed })
    }

    /// Judges `pack`'s licence: allowed when the expression can be satisfied with allowed identifiers
    /// alone; a requirement with a `WITH` addition or `+` is not an allowed identifier.
    ///
    /// # Errors
    ///
    /// [`PackageError::LicenceNotAllowed`], naming the pack, its expression, what failed and the policy.
    pub fn judge(&self, pack: &str, license: &License) -> Result<(), PackageError> {
        let expression =
            spdx::Expression::parse(license.as_str()).map_err(|error| PackageError::License {
                text: license.as_str().to_owned(),
                reason: error.reason.to_string(),
            })?;
        let outcome = expression.evaluate_with_failures(|requirement| {
            requirement.addition.is_none()
                && matches!(
                    &requirement.license,
                    spdx::LicenseItem::Spdx { id, or_later: false } if self.allowed.contains(id.name)
                )
        });
        outcome.map_err(|failed| PackageError::LicenceNotAllowed {
            pack: pack.to_owned(),
            expression: license.as_str().to_owned(),
            failed: failed
                .iter()
                .map(|failure| failure.req.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            allowed: self.allowed.iter().copied().collect::<Vec<_>>().join(", "),
        })
    }
}
