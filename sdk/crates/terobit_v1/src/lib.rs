//! This is the WASM (guest) side SDK for v1.

pub use terobit_v1_macros::main;

pub mod update {
    pub use terobit_v1_macros::{post_physics, post_render, pre_physics, pre_render};
}
