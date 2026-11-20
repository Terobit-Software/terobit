//! This is the WASM (guest) side SDK for v1.

pub use terobit_v1_macros::main;

pub mod update {
    pub use terobit_v1_macros::{post_physics, post_render, pre_physics, pre_render};
}

pub mod frame_info;

mod log;
pub use log::*;

mod window;
pub use window::*;

pub mod prelude;

pub mod ecs;

/// Host functions are wasm imports from `terobit_v1`, which only exist inside the sandbox.
/// Off wasm32 the SDK still compiles (for docs, tests and tooling), but calling one panics.
#[cfg(not(target_arch = "wasm32"))]
#[track_caller]
pub(crate) fn outside_sandbox(function: &str) {
    panic!(
        "`{function}` calls the terobit host, which only exists when the script runs as wasm32 inside terobit"
    )
}
