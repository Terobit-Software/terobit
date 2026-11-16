/// fuel the wasm store starts with, and is refilled to at the end of each update cycle
pub const WASM_FUEL: u64 = 1_000_000;

/// fuel the wasm store is filled to right before an instance's main entrypoint runs
pub const MAIN_WASM_FUEL: u64 = WASM_FUEL * 5;
