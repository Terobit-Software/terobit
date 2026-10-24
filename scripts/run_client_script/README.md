# run_client_script (`cargo rcl`)

Builds the client and runs it from a clean folder.

## Why

Running from `target/` mixes build output with whatever the client writes at runtime.
Copying the binary into a fresh `out/client` means every run starts clean, and the
client's working directory looks like a real install.

## Run

From the workspace root (paths are relative to the current directory):

```sh
cargo rcl            # release-dev build, runs with the dev flags
cargo rcl --final    # release build, runs with no flags
cargo rcl -s         # release-dev build, runs with no flags
```

| flag                     | what it does                                                     |
|--------------------------|------------------------------------------------------------------|
| `--final`, `-f`          | builds with the `release` profile and runs without the dev flags |
| `--strip-defaults`, `-s` | runs without the dev flags                                       |

## What it does

1. `cargo build --profile release-dev -p terobit_client` (`release` with `--final`)
2. deletes and recreates `out/client`
3. copies `bins/terobit_client/assets` to `out/client/assets` if it exists. It doesn't yet,
   so this warns every run
4. copies the built binary into `out/client`
5. runs it from `out/client` with
   `--dev --pack-file <workspace>/tests/examples/source_to_tbpk/basic1`, unless `--final`
   or `-s` is given

The exit code is the client's.

## TODO

- [ ] a `--pack-file <path>` flag so other examples (like generated ones) can run without
  editing the script
- [ ] pass extra arguments through to the client (`cargo rcl -- --some-flag`)
- [ ] build with `--features dev` unless `--final`, once the feature exists (B8)
- [ ] create `bins/terobit_client/assets` or drop step 3
