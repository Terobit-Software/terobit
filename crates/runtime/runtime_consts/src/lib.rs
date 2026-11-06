//! Constants shared across the runtime and its tools. No dependencies, so any
//! crate can use it without pulling anything else in. Client only constants are in
//! `client_consts`.
//!
//! Format constants (`pack`, `integrity`) are part of the on-disk formats in `docs/formats`,
//! changing one is a format version bump.

#![no_std]

pub mod integrity;
pub mod logging;
pub mod pack;
pub mod paths;
pub mod sandbox;
pub mod units;
