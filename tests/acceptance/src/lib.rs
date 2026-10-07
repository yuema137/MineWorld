//! MineWorld's acceptance and conformance tests for its frozen criteria (`docs/ARCHITECTURE.md` §14).
//!
//! This crate holds no code. Its tests, under `tests/`, read the repository and its history rather
//! than one crate's API, because what they check — what a set of pull requests changed, and what it
//! did not — is not a property of any one crate:
//!
//! ```text
//! precursor_vocabulary   I-2: the framework PRs that precede the market name no market concept
//!                        (docs/DECISIONS.md ARC-35, point 7)
//! ```
//!
//! They need the repository's git history and fail, naming what is missing, without it.

#![forbid(unsafe_code)]
