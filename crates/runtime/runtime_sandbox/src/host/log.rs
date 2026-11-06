use wasmtime::{Caller, Linker, Result, bail};

/// links `host::log` (see `terobit_v1_abi::crosspoints::log`).
pub(crate) fn link(linker: &mut Linker<()>) -> Result<()> {
    linker.func_wrap("host", "log", log)?;
    Ok(())
}

fn log(mut caller: Caller<'_, ()>, ptr: i32, len: i32) -> Result<()> {
    let Some(memory) = caller.get_export("memory").and_then(|e| e.into_memory()) else {
        bail!("guest has no exported `memory`");
    };

    let start = ptr as u32 as usize;
    let end = start + len as u32 as usize;
    let Some(bytes) = memory.data(&caller).get(start..end) else {
        bail!("log message {start}..{end} is out of guest memory bounds");
    };

    tracing::info!(target: "wasm", "{}", String::from_utf8_lossy(bytes));
    Ok(())
}
