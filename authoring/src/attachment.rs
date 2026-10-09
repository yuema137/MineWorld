//! Files a configuration names under its World Pack's `data/` directory (`docs/DECISIONS.md` `ARC-61`
//! note, S19's QTW-7).
//!
//! An [`Attachment`] is checked as it decodes, so a path that could leave the pack never reaches the
//! loader: written with `/` whatever the platform, relative, first component `data`, every component a
//! plain name. The loader reads each named file into [`Attached`] and refuses one that is missing, one
//! whose real path lies outside the pack, and one over [`ATTACHMENT_MAX_BYTES`]; the owner receives the
//! bytes when it seeds and states what it needs in its own fact.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The directory attachments live in, and the first component of every attachment's path.
pub const ATTACHMENT_DIRECTORY: &str = "data";

/// The largest attachment the loader reads: 4 MiB (QIB-6).
pub const ATTACHMENT_MAX_BYTES: u64 = 4 * 1024 * 1024;

/// A file under the World Pack's `data/`, as a configuration names it: `data/table.csv`.
///
/// The same text means the same file on macOS, Linux and Windows: components are separated by `/`
/// only, and a `\`, a drive (`:`), an empty component, `.` and `..` are refused rather than given a
/// platform's meaning.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Attachment(Vec<String>);

impl TryFrom<String> for Attachment {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        let refused = |why: &str| format!("the attachment '{text}' {why}");
        if text.contains('\\') {
            return Err(refused("uses '\\'; write paths with '/' on every platform"));
        }
        if text.starts_with('/') {
            return Err(refused("is absolute; name a file under data/"));
        }
        let components: Vec<String> = text.split('/').map(str::to_owned).collect();
        if components.first().map(String::as_str) != Some(ATTACHMENT_DIRECTORY) {
            return Err(refused("is not under data/"));
        }
        if components.len() < 2 {
            return Err(refused("names the data/ directory, not a file in it"));
        }
        for component in &components {
            if component.is_empty() || component == "." || component == ".." {
                return Err(refused("has an empty, '.' or '..' component"));
            }
            if component.contains(':') {
                return Err(refused("names a drive or a stream (':')"));
            }
        }
        Ok(Self(components))
    }
}

impl From<Attachment> for String {
    fn from(attachment: Attachment) -> Self {
        attachment.0.join("/")
    }
}

impl core::fmt::Display for Attachment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0.join("/"))
    }
}

impl Attachment {
    /// The file this names, under a pack's root, in the platform's own form.
    pub fn path_under(&self, root: &Path) -> PathBuf {
        self.0
            .iter()
            .fold(root.to_path_buf(), |path, component| path.join(component))
    }
}

/// The bytes of a configuration's attachments, by the path it named them with. Read-only to the
/// configuration's owner.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attached(BTreeMap<Attachment, Vec<u8>>);

impl Attached {
    /// Nothing attached.
    pub const fn none() -> Self {
        Self(BTreeMap::new())
    }

    /// Records one attachment's bytes (the loader's).
    pub fn insert(&mut self, attachment: Attachment, bytes: Vec<u8>) {
        self.0.insert(attachment, bytes);
    }

    /// The bytes of `attachment`, if it was read.
    pub fn get(&self, attachment: &Attachment) -> Option<&[u8]> {
        self.0.get(attachment).map(Vec::as_slice)
    }

    /// Whether nothing is attached.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attachment(text: &str) -> Result<Attachment, String> {
        Attachment::try_from(text.to_owned())
    }

    /// Only a plain relative path under data/ is an attachment, whatever platform reads it; the
    /// platform's own form is built from its components.
    #[test]
    fn an_attachment_is_a_plain_path_under_data_on_every_platform() {
        for (text, why) in [
            ("/etc/passwd", "is absolute"),
            ("data/../world.yaml", "'..'"),
            ("table.csv", "is not under data/"),
            ("configure/x.yaml", "is not under data/"),
            ("data", "names the data/ directory"),
            ("data//x", "empty"),
            ("data\\table.csv", "uses '\\'"),
            ("C:/data/x", "is not under data/"),
            ("data/C:x", "':'"),
        ] {
            let refusal = attachment(text).expect_err(text);
            assert!(refusal.contains(why), "{text}: {refusal}");
        }
        let table = attachment("data/tables/rows.csv").expect("a plain path");
        assert_eq!(table.to_string(), "data/tables/rows.csv");
        assert_eq!(
            table.path_under(Path::new("pack")),
            Path::new("pack")
                .join("data")
                .join("tables")
                .join("rows.csv")
        );
    }
}
