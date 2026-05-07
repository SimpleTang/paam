//! paam-core — Core business logic for paam (Private AI Asset Manager).
//!
//! Used by both `paam-cli` (CLI entry) and `paam-app` (desktop UI, M3+).
//!
//! See `PRODUCT.md` and `.dev/docs/decisions/` for architectural decisions.

pub mod asset;
pub mod config;
pub mod discover;
pub mod error;
pub mod git;
pub mod install;
pub mod local_repo;
pub mod metadata;
pub mod paths;
pub mod source;
pub mod sync;

#[cfg(test)]
pub(crate) mod test_support;

pub use asset::{Asset, AssetKind};
pub use error::{Error, Result};
pub use paths::PaamRoot;
