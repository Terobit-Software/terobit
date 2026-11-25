use crate::HostState;
use runtime_consts::sandbox::{MAIN_WASM_TICKS, WASM_CYCLE_TICKS};
use snafu::{ResultExt, Snafu};
use terobit_v1_abi::*;

#[derive(Debug, Snafu)]
pub enum InstancePrepError {
    #[snafu(display("no main entrypoint `{}`", endpoints::MAIN_ENTRYPOINT))]
    NoMainEntryPoint { source: wasmtime::Error },

    #[snafu(display("main entrypoint failed"))]
    MainFailed { source: wasmtime::Error },
}

pub fn prep_instance(
    instance: &wasmtime::Instance,
    store: &mut wasmtime::Store<HostState>,
) -> Result<(), InstancePrepError> {
    // --- INIT --- //
    let main_entrypoint = instance
        .get_typed_func::<(), ()>(&mut *store, endpoints::MAIN_ENTRYPOINT)
        .context(NoMainEntryPointSnafu {})?;

    // --- DEADLINE --- //
    store.set_epoch_deadline(MAIN_WASM_TICKS);

    // --- CALL --- //
    let main_result = main_entrypoint
        .call(&mut *store, ())
        .context(MainFailedSnafu {});

    // --- RESET --- //
    // drop back to the per-cycle budget so the longer main deadline doesn't
    // carry over, even if main failed
    store.set_epoch_deadline(WASM_CYCLE_TICKS);

    main_result
}
