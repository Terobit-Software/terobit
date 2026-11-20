//! Functions the HOST provides for the GUEST to call (wasm imports).
//!
//! Every import comes from the [`HOST_MODULE`] module; the host refuses a script that
//! imports from any module it doesn't provide. The extern blocks only exist on wasm32,
//! since they name host functions that don't exist anywhere else.

/// The wasm import module every function here is imported from. The
/// `#[link(wasm_import_module = ...)]` attributes need a literal, so they repeat it.
pub const HOST_MODULE: &str = "terobit_v1";

pub mod commands;
pub mod log;
pub mod window;
