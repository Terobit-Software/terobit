use core::time::Duration;

/// wasm import modules this build of the host provides. a script may only import from these,
/// the pack writer and the loader both refuse anything else (tbman v1, "scripts").
/// a new api version adds its module here, and old ones stay as long as games use them
pub const HOST_MODULES: &[&str] = &["terobit_v1"];

/// how often the engine's epoch is incremented; the granularity of every wasm time budget
pub const WASM_EPOCH_TICK: Duration = Duration::from_millis(1);

/// wall-clock budget for each update cycle (pre-physics, post-physics, pre-render, post-render)
/// and for module instantiation, shared across all instances
pub const WASM_CYCLE_BUDGET: Duration = Duration::from_millis(5000 / 4); // max of 5 seconds across all updates before poison

/// wall-clock budget for an instance's main entrypoint
pub const MAIN_WASM_BUDGET: Duration = Duration::from_millis(5000); // instantiate, max 5 seconds

/// converts a time budget into a number of epoch ticks (rounded up, at least 1)
pub const fn budget_ticks(budget: Duration) -> u64 {
    let tick = WASM_EPOCH_TICK.as_nanos();
    let ticks = budget.as_nanos().div_ceil(tick);
    if ticks == 0 { 1 } else { ticks as u64 }
}

/// epoch ticks for each update cycle
pub const WASM_CYCLE_TICKS: u64 = budget_ticks(WASM_CYCLE_BUDGET);

/// epoch ticks for an instance's main entrypoint
pub const MAIN_WASM_TICKS: u64 = budget_ticks(MAIN_WASM_BUDGET);
