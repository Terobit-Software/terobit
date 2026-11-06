//! Constants for the client and the scripts that build and run it. No dependencies, so any
//! client crate can use it without pulling anything else in. Constants the runtime shares
//! are in `runtime_consts`.

#![no_std]

pub mod flags;
pub mod logging;
pub mod paths;
