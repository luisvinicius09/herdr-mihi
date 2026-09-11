//! Canonical herdr-mihi shared Rust modules — **do not edit inside a plugin**.
//! Source of truth: `shared/rust/`. `just sync-shared` copies these into each plugin's
//! `src/shared/` (any plugin whose src declares `mod shared`), and CI drift-checks the copies.
//! To change shared behavior: edit `shared/rust/`, then `just sync-shared`.
pub mod sanitize;
pub mod snapshot;
pub mod socket;
