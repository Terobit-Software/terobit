# run_client_script (`cargo rcl`)

Builds the client and runs it from a clean folder.

## Why

Running from `target/` mixes build output with whatever the client writes at runtime.
Copying the binary into a fresh `out/client` means every run starts clean, and the
client's working directory looks like a real install.

## Run

From the workspace root (paths are relative to the current directory):

```sh
cargo rcl                    # release-dev build with the dev feature, runs with the dev flags
cargo rcl --final            # release build without the dev feature, runs with no flags
cargo rcl --strip-defaults   # release-dev build with the dev feature, runs with no flags
cargo rcl --pack-file tests/examples/source_to_tbpk/hello_world1   # dev run with another pack
```

| flag                 | what it does                                                     |
|----------------------|------------------------------------------------------------------|
| `--final`            | builds with the `release` profile and without the `dev` feature, and runs without the dev flags |
| `--strip-defaults`   | runs without the dev flags                                       |
| `--pack-file <path>` | the pack or dev directory to load, relative to the current directory. defaults to `tests/examples/source_to_tbpk/basic1`. can't be used with `--final` or `--strip-defaults` |

Flags are read with `RuntimeFlags`, so they're long only.

## Testing an SDK script

The `hello_world1` pack runs `tests/examples/sdk/hello_world1` as its client script. The
`.wasm` is gitignored, so build and copy it in first:

```sh
cargo build -p hello_world1 --target wasm32-unknown-unknown --profile release-dev
mkdir -p tests/examples/source_to_tbpk/hello_world1/scripts
cp target/wasm32-unknown-unknown/release-dev/hello_world1.wasm tests/examples/source_to_tbpk/hello_world1/scripts/client.wasm
cargo rcl --pack-file tests/examples/source_to_tbpk/hello_world1
```

## What it does

1. `cargo build --profile release-dev -p terobit_client --features dev`, or
   `cargo build --profile release -p terobit_client` with `--final`
2. deletes and recreates `out/client`
3. copies `bins/terobit_client/assets` to `out/client/assets` if it exists. It doesn't yet,
   so this warns every run
4. copies the built binary into `out/client`
5. runs it from `out/client` with
   `--dev --pack-file <workspace>/<pack file>`, unless `--final` or `--strip-defaults` is
   given

The exit code is the client's.

## TODO

- [x] a `--pack-file <path>` flag so other examples (like generated ones) can run without
  editing the script
- [ ] build SDK scripts and copy them into their pack before running
- [ ] pass extra arguments through to the client (`cargo rcl -- --some-flag`)
- [ ] create `bins/terobit_client/assets` or drop step 3
