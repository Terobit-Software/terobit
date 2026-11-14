# xtask (`cargo xtask`, `cargo x`)

Workspace tasks, run from the workspace root (paths are relative to the current directory).
Arguments are parsed with clap, so every command has `--help`.

```sh
cargo xtask --help
cargo x run client --help
```

## run client (`cargo x r cl`)

Builds the client and runs it from a clean folder. `r` is short for `run` and `cl` for
`client`, so `cargo x r cl` is the same as `cargo xtask run client`.

Running from `target/` mixes build output with whatever the client writes at runtime.
Copying the binary into a fresh `out/client` means every run starts clean, and the
client's working directory looks like a real install.

```sh
cargo x run client                    # release-dev build with the dev feature, runs with the dev flags
cargo x run client --final            # release build without the dev feature, runs with no flags
cargo x run client --strip-defaults   # release-dev build with the dev feature, runs with no flags
cargo x run client --pack-file tests/examples/source_to_tbpk/hello_world1   # dev run with another pack
```

| flag                 | what it does                                                     |
|----------------------|------------------------------------------------------------------|
| `--final`            | builds with the `release` profile and without the `dev` feature, and runs without the dev flags |
| `--strip-defaults`   | runs without the dev flags                                       |
| `--pack-file <path>` | the pack or dev directory to load, relative to the current directory. defaults to `tests/examples/source_to_tbpk/basic1`. can't be used with `--final` or `--strip-defaults` |

### Testing an SDK script

The `hello_world1` pack runs `tests/examples/sdk/hello_world1` as its client script. The
`.wasm` is gitignored, so build and copy it in first:

```sh
cargo build -p hello_world1 --target wasm32-unknown-unknown --profile release-dev
mkdir -p tests/examples/source_to_tbpk/hello_world1/scripts
cp target/wasm32-unknown-unknown/release-dev/hello_world1.wasm tests/examples/source_to_tbpk/hello_world1/scripts/client.wasm
cargo x run client --pack-file tests/examples/source_to_tbpk/hello_world1
```

### What it does

1. `cargo build --profile release-dev -p terobit_client --features dev`, or
   `cargo build --profile release -p terobit_client` with `--final`
2. deletes and recreates `out/client`
3. copies `bins/terobit_client/assets` to `out/client/assets` if it exists. It doesn't yet,
   so this warns every run
4. copies the built binary into `out/client`
5. runs it from `out/client` with
   `--dev --pack-file <workspace>/<pack file>`, unless `--final` or `--strip-defaults` is
   given

Exits with failure if the build or the client fails.

## pkgen

`cargo x pkgen` runs [tbpk_generator](../bins/tbpk_generator/README.md), built with the
`release` profile. Every argument after `pkgen` is passed through unchanged, and
tbpk_generator reads them with `RuntimeFlags`, so see its README for the flags.

```sh
cargo x pkgen --manifest tests/examples/source_to_tbpk/basic1/manifest.toml
```

## regen

Writes example dev directories full of generated files, for testing and benchmarking
`cargo x pkgen` and the chunker on packs bigger than the hand-made examples.

```sh
cargo x regen                                  # every type at its default count
cargo x regen --type mix --count 500 --auto    # one example, then pack it
cargo x regen --type random --min 1 --max 65536 --entropy med
```

| flag        | value                                                      | default            |
|-------------|------------------------------------------------------------|--------------------|
| `--type`    | `xsmall`, `small`, `large`, `xlarge`, `mix`, `xmix`, `random` | all of them     |
| `--count`   | files per side, shared files count for both. Needs `--type` | per type, below   |
| `--min`     | smallest file in bytes, `--type random` only               | 1                  |
| `--max`     | largest file in bytes, `--type random` only                | 1 MiB              |
| `--entropy` | `low`, `med`, `high`: how well the files compress          | `high`             |
| `--auto`    | pack each example with tbpk_generator afterwards           | off                |

| type     | file sizes                 | default count |
|----------|----------------------------|---------------|
| `xsmall` | 1 - 64 B                   | 500           |
| `small`  | 65 B - 4 KiB               | 250           |
| `large`  | 256 KiB + 1 B - 4 MiB      | 25            |
| `xlarge` | 8 - 16 MiB                 | 25            |
| `mix`    | `small` and `large`        | 100           |
| `xmix`   | every class above          | 100           |
| `random` | `--min` to `--max`         | 50            |

Examples go to `tests/examples/source_to_tbpk/generated/<name>/`, which is gitignored.
The name comes from the settings (like `mix_100` or `small_250_low`), so the same settings
always map to the same folder. An example that already exists is skipped, so delete its
folder to regenerate it. With `--auto`, packs go to `out/tbpk_generator/<name>/`. If any
example fails, the rest still run and the task exits with failure.

### How it works

- about half the files are shared (`[assets]`), the rest are split between
  `[client.assets]` and `[server.assets]`, 20 files per folder
- file bytes come from a blake3 stream keyed by the example's settings and the file's
  path, so the same settings always produce the same files. The key is still
  `terobit example_generator` from before regen moved into xtask, so old examples match
- each example is written to `<name>.tmp` and renamed when it's done, so a half-written
  example is never mistaken for a finished one

## TODO

- [ ] build SDK scripts and copy them into their pack before running
- [ ] pass extra arguments through to the client (`cargo x run client -- --some-flag`)
- [ ] create `bins/terobit_client/assets` or drop step 3
- [ ] regen: rewrite it, its module doc says so (C7)
- [ ] regen: add a size class between 4 KiB and 256 KiB. Those are the files that share
  small-file chunks, and only `--type random` makes them now (C7)
