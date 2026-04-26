//! paam-core — Core business logic for paam (Private AI Asset Manager).
//!
//! Used by both `paam-cli` (CLI entry) and `paam-app` (desktop UI, M3+).
//!
//! See `PRODUCT.md` and `.dev/docs/decisions/` for architectural decisions.

// M1 modules will be added incrementally:
//   config       — config.json read/write
//   git          — git2-rs wrapper
//   source       — track / asset discovery
//   local_repo   — install / auto-commit
//   sync         — symlink / conflict detection
//   metadata     — .metadata.json read/write
//   asset        — Asset trait + Skill impl
