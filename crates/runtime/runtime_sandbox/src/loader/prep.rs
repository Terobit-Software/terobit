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
    store: &mut wasmtime::Store<()>,
) -> Result<(), InstancePrepError> {
    // --- INIT --- //
    let main_entrypoint = instance
        .get_typed_func::<(), ()>(&mut *store, endpoints::MAIN_ENTRYPOINT)
        .context(NoMainEntryPointSnafu {})?;

    // --- CALL --- //
    main_entrypoint
        .call(&mut *store, ())
        .context(MainFailedSnafu {})?;

    Ok(())
}
