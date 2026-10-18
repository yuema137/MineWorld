//! Which section of authored content a System Pack owns (`DECISIONS.md` `ARC-31`), and which data
//! directories (`ARC-121`).

use mineworld_authoring::{
    AuthoredSection, ContentKind, DataDirectory, OwnsData, SectionInContext, SectionName,
};

/// The section of authored content a pack owns: its key, and the files that may carry it
/// (`DECISIONS.md` `ARC-31`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionOwner {
    /// The key.
    pub name: SectionName,
    /// The kinds of content file that may carry it.
    pub carried_by: &'static [ContentKind],
}

impl SectionOwner {
    /// The section `S` declares, read off its [`AuthoredSection`] impl so the two cannot disagree.
    /// Used by [`owns_section!`](crate::owns_section).
    pub const fn of<S: AuthoredSection>() -> Self {
        Self {
            name: S::SECTION,
            carried_by: S::CARRIED_BY,
        }
    }

    /// The section `S` declares, read off its [`SectionInContext`] impl (`ARC-121` item 2). Used by
    /// [`owns_section_in_context!`](crate::owns_section_in_context).
    pub const fn of_in_context<S: SectionInContext>() -> Self {
        Self {
            name: S::SECTION,
            carried_by: S::CARRIED_BY,
        }
    }

    /// Whether a file of this kind may carry the section.
    pub fn carried_by(&self, kind: ContentKind) -> bool {
        self.carried_by.contains(&kind)
    }
}

/// The data directories a pack owns (`DECISIONS.md` `ARC-121` item 1), in the order their files are
/// handed over within one source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataOwnerDecl {
    /// The directories.
    pub directories: &'static [DataDirectory],
}

impl DataOwnerDecl {
    /// The directories `S` declares, read off its [`OwnsData`] impl. Used by
    /// [`owns_data!`](crate::owns_data).
    pub const fn of<S: OwnsData>() -> Self {
        Self {
            directories: S::DIRECTORIES,
        }
    }

    /// The owned directory written `name`, if this pack owns one.
    pub fn directory(&self, name: &str) -> Option<DataDirectory> {
        self.directories
            .iter()
            .copied()
            .find(|directory| directory.as_str() == name)
    }
}
