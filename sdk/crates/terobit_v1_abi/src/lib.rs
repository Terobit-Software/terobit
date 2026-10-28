//! This crate defines the types, variables, and conventions
//! passed between the HOST (wasmtime) and the GUEST (wasm/sdk).

// this is important, because std is not in languages other than rust
#![no_std]

pub mod endpoints;
pub mod info_structs;
