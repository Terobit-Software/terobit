use terobit_v1_abi::crosspoints::HOST_MODULE;
use wasmtime::{Caller, Linker, Result, bail};

use crate::{
    HostState,
    commands::{apply_commands, read_commands_from_memory},
};

/// links `terobit_v1::flush_commands` (see `terobit_v1_abi::crosspoints::commands`).
pub(crate) fn link(linker: &mut Linker<HostState>) -> Result<()> {
    linker.func_wrap(HOST_MODULE, "flush_commands", flush_commands)?;
    Ok(())
}

fn flush_commands(mut caller: Caller<'_, HostState>, ptr: i32, len: i32) -> Result<()> {
    let Some(memory) = caller.get_export("memory").and_then(|e| e.into_memory()) else {
        bail!("guest has no exported `memory`");
    };
    let guest_commands = read_commands_from_memory(memory.data(&caller), ptr, len)?;

    let Some(mut world) = caller.data().world else {
        bail!("no world access");
    };
    let world = unsafe { world.as_mut() };

    apply_commands(&mut world.commands(), &guest_commands);
    world.flush();
    Ok(())
}
