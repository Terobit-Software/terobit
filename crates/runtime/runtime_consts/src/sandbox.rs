use core::time::Duration;

/// how often the engine's epoch is incremented; the granularity of every wasm time budget
pub const WASM_EPOCH_TICK: Duration = Duration::from_millis(1);

/// wall-clock budget for each update cycle (pre-physics, post-physics, pre-render, post-render)
/// and for module instantiation, shared across all instances
pub const WASM_CYCLE_BUDGET: Duration = Duration::from_millis(60);

/// wall-clock budget for an instance's main entrypoint
pub const MAIN_WASM_BUDGET: Duration = Duration::from_millis(500);

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
