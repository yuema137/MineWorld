//! A town's far field fills its slots with the places the town declares (`DECISIONS.md` `ARC-107`,
//! `MODULE_SPEC.md` §6.1.1, "Town slots").
//!
//! The rule needs the world's `settlement:`, which no client reads, so it runs at the authoring gate
//! (`mineworld validate` and `mineworld packs validate`) and never in `run`, `server` or the loader. It
//! visits every 3D far field that presents this world: in each Presentation Pack the world requires, then
//! in each one the pack roots hold, in a fixed order with no repeats. The place keys come from
//! `settlement:` alone; nothing here names a world or a place.

use std::path::PathBuf;

use mineworld_packages::far_field::{self, FAR_FIELD_EXTENSION};
use mineworld_packages::{
    DataPack, Dimension, FAR_FIELD_DIR, PackRoots, PackType, PackageError, SlotRule, Source,
    check_slots, check_style_manifest, read_pack_file,
};

use crate::error::PackError;
use crate::read::{MANIFEST, WorldPack};
use crate::settlement::SettlementKind;

impl WorldPack {
    /// Checks every 3D far field presenting this world against its `settlement:` (`ARC-107`). Does nothing
    /// when the world declares no settlement: `require_settlement` refuses that first at the gate.
    ///
    /// # Errors
    ///
    /// The first far field that disagrees, as `PackageError::FarFieldSlots` (or the file's own refusal),
    /// wrapped as a requirements refusal of this world's `world.yaml`.
    pub fn check_far_field_slots(&self, roots: &PackRoots) -> Result<(), PackError> {
        let Some(settlement) = self.settlement() else {
            return Ok(());
        };
        let rule = match settlement.kind() {
            SettlementKind::Town { station, landmark } => SlotRule::Town {
                station: station.as_str(),
                landmark: landmark.as_str(),
            },
            SettlementKind::None => SlotRule::NotATown,
        };
        let refused = |refusal| PackError::Requirements {
            path: self.root().join(MANIFEST),
            refusal: Box::new(refusal),
        };
        for (pack, dir) in self.presentation_packs(roots).map_err(refused)? {
            let dimensions = check_style_manifest(&dir).map_err(refused)?;
            if !dimensions.contains(&Dimension::ThreeD) {
                continue;
            }
            let path = dir
                .join(FAR_FIELD_DIR)
                .join(format!("{}.{FAR_FIELD_EXTENSION}", self.id()));
            if !path.is_file() {
                continue;
            }
            let field = far_field::read(&path, &dimensions).map_err(refused)?;
            check_slots(&field, rule).map_err(|(code, slot, detail)| {
                refused(PackageError::FarFieldSlots {
                    code,
                    path: path.clone(),
                    pack: pack.clone(),
                    world: self.id().to_owned(),
                    slot,
                    detail,
                })
            })?;
        }
        Ok(())
    }

    /// The Presentation Packs that may present this world, by id and directory: the required ones in id
    /// order, then those the roots hold in root order, each directory once.
    fn presentation_packs(
        &self,
        roots: &PackRoots,
    ) -> Result<Vec<(String, PathBuf)>, PackageError> {
        let mut packs: Vec<(String, PathBuf)> = Vec::new();
        let mut seen: Vec<PathBuf> = Vec::new();
        let mut add = |id: String, dir: PathBuf, packs: &mut Vec<(String, PathBuf)>| {
            let canonical = dir.canonicalize().unwrap_or_else(|_| dir.clone());
            if !seen.contains(&canonical) {
                seen.push(canonical);
                packs.push((id, dir));
            }
        };
        for required in &self.composition().required {
            if let (PackType::PresentationPack, Source::Directory(dir)) =
                (required.identity.kind, &required.source)
            {
                add(required.identity.id.to_string(), dir.clone(), &mut packs);
            }
        }
        for found in roots.packs()? {
            let DataPack::PackFile(dir) = found else {
                continue;
            };
            let identity = read_pack_file(&dir)?;
            if identity.kind == PackType::PresentationPack {
                add(identity.id.to_string(), dir, &mut packs);
            }
        }
        Ok(packs)
    }
}
