# tbpk_generator

Builds `.tbpk` packs from a dev directory. It stands in for the editor's publish step until
the editor exists.

## Why

The platform takes archives, and released clients will only load archives. This turns a
`manifest.toml` and the files it lists into one client pack and one server pack, using the
same `runtime_assets` code that will read them, so the format only has one implementation.

## Run

From the workspace root:

```sh
cargo pkgen --manifest tests/examples/source_to_tbpk/basic1/manifest.toml
```

| flag           | value                    | default                 | notes                                                     |
|----------------|--------------------------|-------------------------|-----------------------------------------------------------|
| `--manifest`   | path to `manifest.toml`  | required                | asset paths are relative to its folder                    |
| `--out`        | folder                   | `out/tbpk_generator`    | created if missing                                        |
| `--target`     | `client`, `server`, `both` | `both`                |                                                           |
| `--zstd-level` | a number, or `none`      | `ZSTD_LEVEL` (5)        | any other value changes chunk bytes and hashes, so only use it to test speed and size |

Each flag takes exactly one value. There is no `--help` yet (C6).

Output is `<out>/<realm_id>-<pack_version>.client.tbpk` and `.server.tbpk`. The example
above writes `out/tbpk_generator/1-1.client.tbpk` and `1-1.server.tbpk`.

Logs go to the terminal and to a daily `tbpk-generator.log.<date>` file in
`~/.local/share/tbpk/generator/logs/` on Linux, `~/Library/Logs/Tbpk/Generator/` on macOS,
or `%LOCALAPPDATA%\tbpk\generator\logs\` on Windows.

## How it works

1. a minimal Bevy app runs one startup system, so it can reuse `RuntimeRunFlagsPlugin` and
   Bevy's exit codes
2. the flags are parsed into `Args`. A bad flag exits with an error before anything is read
3. `PackWriter::new().with_zstd_level(...)`, then `load_manifest`, which validates the
   manifest
4. for each target, `build_tbpk` then `write_tbpk_out`. If one target fails it's logged and
   the other still builds. The exit code is non-zero if any target failed

See [building a pack](../../docs/ARCHITECTURE.md#building-a-pack) for what `PackWriter`
does inside.

## State

- writes valid, unsigned v1 archives
- no signing key loading, so packs are always unsigned (F2)
- prints a few hundred lines even for a tiny pack (C1)

## TODO

- [ ] load a signing key and certificate, pass them with `with_signer` (F2)
- [ ] after writing, read the pack back and compare every file, once the reader exists (F1)
- [ ] switch to clap so `--help` works (C6)
- [ ] `--chunk-size` for testing, once the writer supports it (F10)
