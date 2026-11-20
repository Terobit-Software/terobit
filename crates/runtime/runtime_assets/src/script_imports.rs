//! checks a script only imports from host modules terobit provides.
//!
//! tbman v1 says a script may only import from versioned host modules (`terobit_v1`, ...),
//! no wasi and nothing else. the pack writer runs this before packing a script and the
//! sandbox runs it again before compiling one, so a bad import is caught with a clear
//! error instead of a link failure

use runtime_consts::sandbox::HOST_MODULES;
use snafu::{ResultExt, Snafu, ensure};

/// a script's imports couldn't be read, or it imports something the host doesn't provide
#[derive(Debug, Snafu)]
pub enum ScriptImportError {
    #[snafu(display("not a valid wasm module"))]
    Parse {
        source: wasmparser::BinaryReaderError,
    },

    #[snafu(display("is a wasm component, but scripts must be core wasm modules"))]
    Component,

    #[snafu(display(
        "imports `{module}::{name}`, but scripts may only import from {}",
        allowed_modules()
    ))]
    ForeignImport { module: String, name: String },
}

fn allowed_modules() -> String {
    HOST_MODULES
        .iter()
        .map(|m| format!("`{m}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// fails on the first import whose module isn't in [`HOST_MODULES`]
pub fn check_script_imports(wasm: &[u8]) -> Result<(), ScriptImportError> {
    for payload in wasmparser::Parser::new(0).parse_all(wasm) {
        match payload.context(ParseSnafu)? {
            wasmparser::Payload::Version {
                encoding: wasmparser::Encoding::Component,
                ..
            } => return ComponentSnafu.fail(),
            wasmparser::Payload::ImportSection(reader) => {
                for import in reader.into_imports() {
                    let import = import.context(ParseSnafu)?;
                    ensure!(
                        HOST_MODULES.contains(&import.module),
                        ForeignImportSnafu {
                            module: import.module,
                            name: import.name,
                        }
                    );
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// a module importing one `fn()` from each `(module, name)`
    fn module_importing(imports: &[(&str, &str)]) -> Vec<u8> {
        let mut out = b"\0asm\x01\0\0\0".to_vec();
        // type section: one `fn() -> ()`
        out.extend_from_slice(&[0x01, 0x04, 0x01, 0x60, 0x00, 0x00]);
        let mut body = vec![imports.len() as u8];
        for (module, name) in imports {
            body.push(module.len() as u8);
            body.extend_from_slice(module.as_bytes());
            body.push(name.len() as u8);
            body.extend_from_slice(name.as_bytes());
            body.extend_from_slice(&[0x00, 0x00]); // func, type 0
        }
        out.push(0x02);
        out.push(body.len() as u8);
        out.extend(body);
        out
    }

    #[test]
    fn accepts_terobit_v1() {
        let wasm = module_importing(&[("terobit_v1", "log"), ("terobit_v1", "set_title")]);
        check_script_imports(&wasm).unwrap();
    }

    #[test]
    fn accepts_no_imports() {
        check_script_imports(&module_importing(&[])).unwrap();
    }

    #[test]
    fn refuses_other_modules() {
        for module in [
            "host",
            "env",
            "wasi_snapshot_preview1",
            "terobit_v2",
            "terobit_v1x",
        ] {
            let wasm = module_importing(&[("terobit_v1", "log"), (module, "f")]);
            match check_script_imports(&wasm) {
                Err(ScriptImportError::ForeignImport { module: m, name }) => {
                    assert_eq!((m.as_str(), name.as_str()), (module, "f"));
                }
                other => panic!("`{module}` should be refused, got {other:?}"),
            }
        }
    }

    #[test]
    fn refuses_garbage() {
        assert!(matches!(
            check_script_imports(b"not wasm"),
            Err(ScriptImportError::Parse { .. })
        ));
    }
}
