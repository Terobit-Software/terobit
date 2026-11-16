use runtime_consts::sandbox::{MAIN_WASM_FUEL, WASM_FUEL};
use snafu::{ResultExt, Snafu};
use terobit_v1_abi::*;

#[derive(Debug, Snafu)]
pub enum InstancePrepError {
    #[snafu(display("no main entrypoint `{}`", endpoints::MAIN_ENTRYPOINT))]
    NoMainEntryPoint { source: wasmtime::Error },

    #[snafu(display("failed to fuel main entrypoint"))]
    FuelMain { source: wasmtime::Error },

    #[snafu(display("main entrypoint failed"))]
    MainFailed { source: wasmtime::Error },

    #[snafu(display("failed to refill fuel after main entrypoint"))]
    Refill { source: wasmtime::Error },
}

pub fn prep_instance(
    instance: &wasmtime::Instance,
    store: &mut wasmtime::Store<()>,
) -> Result<(), InstancePrepError> {
    // --- INIT --- //
    let main_entrypoint = instance
        .get_typed_func::<(), ()>(&mut *store, endpoints::MAIN_ENTRYPOINT)
        .context(NoMainEntryPointSnafu {})?;

    // --- FUEL --- //
    store
        .set_fuel(MAIN_WASM_FUEL)
        .context(FuelMainSnafu {})?;

    // --- CALL --- //
    let main_result = main_entrypoint
        .call(&mut *store, ())
        .context(MainFailedSnafu {});

    // --- REFILL --- //
    // reset to the per-cycle budget so leftover main fuel doesn't carry over,
    // even if main failed
    store.set_fuel(WASM_FUEL).context(RefillSnafu {})?;

    main_result
}
