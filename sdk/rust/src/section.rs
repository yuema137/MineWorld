//! Which section of authored content a System Pack owns (`DECISIONS.md` `ARC-31`).

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName};

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

    /// Whether a file of this kind may carry the section.
    pub fn carried_by(&self, kind: ContentKind) -> bool {
        self.carried_by.contains(&kind)
    }
}
