//! Every way a package identity is refused, each naming what was wrong (`ARC-53`).

use std::path::PathBuf;

/// A package fact that is refused. Never ignored: each variant names the value, the field or the
/// file, so the person who wrote it can find it.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PackageError {
    /// An id outside the rule.
    #[error("'{text}' is not a pack id: {why}")]
    Id {
        /// The id as written.
        text: String,
        /// Which part of the rule it breaks.
        why: &'static str,
    },
    /// A type no pack has.
    #[error("'{text}' is not a pack type: {why}")]
    Type {
        /// The type as written.
        text: String,
        /// Which types there are, or why this one is not read.
        why: &'static str,
    },
    /// A version that is not semver.
    #[error("'{text}' is not a semver version (MAJOR.MINOR.PATCH): {reason}")]
    Version {
        /// The version as written.
        text: String,
        /// The parser's reason.
        reason: String,
    },
    /// A range that is not a semver range.
    #[error("'{text}' is not a semver range: {reason}")]
    Range {
        /// The range as written.
        text: String,
        /// The parser's reason.
        reason: String,
    },
    /// A licence that is not an SPDX expression.
    #[error("'{text}' is not an SPDX licence expression: {reason}")]
    License {
        /// The licence as written.
        text: String,
        /// The parser's reason.
        reason: String,
    },
    /// No author, or an empty one.
    #[error("no author is stated: provenance needs at least one, none of them empty")]
    NoAuthors,
    /// A framework range the running framework does not satisfy.
    #[error("mineworld: \"{range}\" does not admit this framework's version, {framework}")]
    FrameworkNotSupported {
        /// The range as written.
        range: String,
        /// The framework's version.
        framework: String,
    },
    /// A code pack whose Cargo fields do not make an identity.
    #[error("the code pack {pack}: {problem}")]
    CodePack {
        /// The pack's Cargo name.
        pack: String,
        /// What is wrong with it.
        problem: Box<PackageError>,
    },
    /// A data pack missing a package field its carrier must state.
    #[error("{path}: {field} is required by `mineworld packs validate` and is not stated")]
    Missing {
        /// The file that should state it.
        path: PathBuf,
        /// The field, as written in that file.
        field: &'static str,
    },
    /// A file that does not read as its format, with the parser's position where it has one.
    #[error("{path}: {message}")]
    Malformed {
        /// The file.
        path: PathBuf,
        /// The parser's message.
        message: String,
    },
    /// A `type` that `pack.yaml` does not carry.
    #[error("{path}: type {kind} is not stated in pack.yaml — {why}")]
    NotCarriedHere {
        /// The `pack.yaml`.
        path: PathBuf,
        /// The type it stated.
        kind: String,
        /// Where that type is identified instead, or why it is not read.
        why: &'static str,
    },
    /// An Entity Pack holding a directory of content it may not carry in MVP-0 (`ARC-71`).
    #[error(
        "{path}: an Entity Pack carries item kinds only in MVP-0 — places, people and organizations \
         belong in a World Pack"
    )]
    EntityPackCarries {
        /// The directory it may not have.
        path: PathBuf,
    },
    /// An Entity Pack with no item kind (`ARC-71`).
    #[error(
        "{dir}: its pack.yaml states an entity-pack, which declares nothing — it needs items/<key>.yaml, \
         one file per item kind"
    )]
    EntityPackDeclaresNothing {
        /// The pack's directory.
        dir: PathBuf,
    },
    /// A data pack whose own `mineworld:` range the running framework is not in (`ARC-54` note).
    #[error(
        "{id} states mineworld: \"{range}\", which does not admit this framework's version, {framework}"
    )]
    PackFrameworkNotSupported {
        /// The pack's id.
        id: String,
        /// The range as written.
        range: String,
        /// The framework's version.
        framework: String,
    },
    /// A directory holding both manifests.
    #[error("{dir} holds both world.yaml and pack.yaml: a pack has one manifest")]
    TwoManifests {
        /// The directory.
        dir: PathBuf,
    },
    /// A directory holding no manifest, offered as a pack.
    #[error("{dir} is not a pack: it holds neither world.yaml nor pack.yaml")]
    NotAPack {
        /// The directory.
        dir: PathBuf,
    },
    /// A directory of packs that is itself a pack.
    #[error(
        "{dir} is a pack (it holds {file}), not a directory of packs: use `mineworld packs validate {dir}`"
    )]
    RootIsAPack {
        /// The directory.
        dir: PathBuf,
        /// The manifest it holds.
        file: &'static str,
    },
    /// A directory or file that could not be read at all.
    #[error("{path}: could not be read: {reason}")]
    Unreadable {
        /// What could not be read.
        path: PathBuf,
        /// The operating system's reason.
        reason: String,
    },
    /// Two packs with one id.
    #[error("two packs have the id {id}: {first} and {second}")]
    DuplicateId {
        /// The id.
        id: String,
        /// Where the first one comes from.
        first: String,
        /// Where the second one comes from.
        second: String,
    },
    /// A pack root that is not a directory.
    #[error("the pack directory {path} (from {source_name}) does not exist or is not a directory")]
    NoSuchRoot {
        /// The path given.
        path: PathBuf,
        /// Where it was given: `--packs` or `MINEWORLD_PACKS`.
        source_name: &'static str,
    },
    /// A required pack found nowhere (`ARC-54`).
    #[error("requires: {id} is not in {searched}")]
    Absent {
        /// The required id.
        id: String,
        /// Every place searched, or that no pack directory was given.
        searched: String,
    },
    /// A required pack of a type a world cannot require.
    #[error("requires: {id} is a {kind}, which a world cannot require — {why}")]
    WrongType {
        /// The required id.
        id: String,
        /// Its type.
        kind: String,
        /// Why.
        why: &'static str,
    },
    /// A bundled pack listed in `requires:`.
    #[error(
        "requires: {id} is bundled — versioned with the framework, which `mineworld:` already \
         covers; remove it from requires"
    )]
    BundledRequired {
        /// The required id.
        id: String,
    },
    /// A required pack at a version the range does not admit.
    #[error("requires: {id} \"{range}\", but the {id} found is {found}")]
    OutOfRange {
        /// The required id.
        id: String,
        /// The range required.
        range: String,
        /// The version found.
        found: String,
    },
    /// An enabled third-party system whose pack is not required.
    #[error(
        "systems: enables {system}, whose pack {pack} is third-party; add `{pack}: \"<range>\"` to \
         requires"
    )]
    ThirdPartyNotRequired {
        /// The system id.
        system: String,
        /// Its pack's id.
        pack: String,
    },
    /// A licence the policy does not allow (`ARC-55`).
    #[error(
        "{pack}'s licence \"{expression}\" is not allowed: {failed} cannot be satisfied by the licence \
         policy ({allowed})"
    )]
    LicenceNotAllowed {
        /// The pack.
        pack: String,
        /// Its expression.
        expression: String,
        /// The requirements that failed, as written.
        failed: String,
        /// The allowed identifiers.
        allowed: String,
    },
}
