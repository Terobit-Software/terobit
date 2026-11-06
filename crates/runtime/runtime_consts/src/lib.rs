//! Constants shared across the runtime, its binaries and scripts. No dependencies, so any
//! crate can use it without pulling anything else in.
//!
//! Format constants (`pack`, `integrity`) are part of the on-disk formats in `docs/formats`,
//! changing one is a format version bump.

#![no_std]

pub mod units;
pub mod pack;
pub mod integrity;
pub mod flags;
pub mod sandbox;
pub mod logging;
pub mod paths;
