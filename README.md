# Terobit
The engine repo for the Terobit project. This does NOT contain the website. That is hosted on a separate repo for security reasons.

New here? Read [ARCHITECTURE](docs/ARCHITECTURE.md) for how the pieces fit together, and [TODO](TODO.md) for what's broken, missing and next.

## Getting started

You need [rustup](https://rustup.rs). The first build downloads the toolchain targets listed in [rust-toolchain.toml](rust-toolchain.toml).

On Linux, install the system libraries Bevy needs (the same ones CI installs):

```sh
sudo apt-get install libudev-dev libasound2-dev libxkbcommon-dev libwayland-dev
```

Then, from the repo root:

```sh
cargo test --workspace                                                   # run the tests
cargo pkgen --manifest tests/examples/source_to_tbpk/basic1/manifest.toml  # build example packs
cargo rcl                                                                # build and run the client
```

CI runs `cargo fmt --all --check`, `cargo check --workspace --all-targets` and the tests with [nextest](https://nexte.st). Run `cargo fmt --all` before pushing.

## Commands

### cargo rcl
Build and run the client. builds `terobit_client`, copies it and `bins/terobit_client/assets` (if it exists) into a fresh `out/client`, and runs it from there with `--dev --pack-file <repo>/tests/examples/source_to_tbpk/basic1`. See [README](scripts/run_client_script/README.md).

|args|desc|
|----|----|
|--final, -f|builds with the release profile instead of release-dev, and runs without the dev flags|
|--strip-defaults, -s|runs without the default dev flags (`--dev`, `--pack-file ...`)|

### cargo pkgen
Build .tbpk packs from a manifest.toml. packs are named `<realm_id>-<pack_version>.<client\|server>.tbpk`. packs are unsigned for now. See [README](bins/tbpk_generator/README.md).

|args|desc|
|--|--|
|--manifest|path to the manifest.toml, required. asset paths are relative to its folder|
|--out|folder the packs are written to, created if missing. defaults to `out/tbpk_generator`|
|--target|client, server, or both. defaults to both|
|--zstd-level|zstd level chunks are compressed at, or `none` to store them uncompressed. defaults to `ZSTD_LEVEL`, any other level changes chunk bytes and hashes|

### cargo regen
Generate example dev directories in `tests/examples/source_to_tbpk/generated`, for test and example packs. existing examples are skipped. See [README](scripts/example_generator/README.md).

|args|desc|
|--|--|
|--type|xsmall, small, large, xlarge, mix, xmix, or random. without it, every type is generated at its default count|
|--count|files per side (shared files count for both), needs --type. defaults per type|
|--min|smallest file size in bytes, only with `--type random`. defaults to 1|
|--max|largest file size in bytes, only with `--type random`. defaults to 1 MiB|
|--entropy|how random the file bytes are, so how well they compress: low, med, high. defaults to high|
|--auto|runs pkgen on each example afterwards, packs go to `out/tbpk_generator/<example>`|

## Layout
Here is the outline of the layout for the project:

<details>
<summary>Click here to expand</summary>

### /.cargo
Contains the alias configs in [config.toml](.cargo/config.toml). The aliases are outlined in [Commands](#commands)

### /.github
Contains dependabot config and CI config for github.

### /bins
Contains binaries for the repo. See [binaries](#binaries).

### /crates
Contains all the crates for the repo. `/crates/runtime` holds crates any binary can use, and `/crates/{BINARY}` (like `/crates/client`) holds crates that belong to one binary. It is best practice to split binaries into many small, independent crates. See [crates](docs/ARCHITECTURE.md#crates) for what each one does.

### /docs
Contains the documentation for the project. [ARCHITECTURE](docs/ARCHITECTURE.md) explains how everything fits together. In `/docs/formats` is the documentation for custom file types.

### /out
A temporary directory for binary outputs. Also includes `.tbpk` files in `/out/tbpk_generator` usually.

### /scripts
Contains scripts run in some of the [aliases](#commands). 

### /tests
The directory containing tests and examples for local testing. `/tests/examples/source_to_tbpk` holds dev directories (`basic1`, `many_small1`, and `basic1_should_fail`, which must fail to load). In `/tests/examples/source_to_tbpk/generated` are generated source `manifest.toml`s (gitignored, made by `cargo regen`).

### /.gitignore
Gitignore.

### /Cargo.lock
The cargo lock file.

### /Cargo.toml
Contains definition and dependencies for the workspace.

### /CONVENTIONS.md
Contains conventions and rules for editing the project.

### /LICENSE
Contains the **source available** license for the project.

### /README.md
This file.

### /rust-toolchain.toml
Contains build targets for the project.

### /TODO.md
What's broken, missing and out of date, with why and how for each item.

</details>

## Binaries

### Pack Generator (tbpk_generator)
Generates .tbpk files from a manifest and asset directory. Working, writes unsigned packs. See [README](bins/tbpk_generator/README.md).

### Client (terobit_client)
Runs and parses .client.tbpk files. Currently opens an empty window. With `--dev` it loads a dev directory's assets, but nothing uses them yet. See [README](bins/terobit_client/README.md).

### Editor (terobit_editor)
Runs, parses, edits, and creates .tbpk files. The main editing suite for Terobit realms. Not started, tbpk_generator and `cargo rcl` stand in for it. See [README](bins/terobit_editor/README.md).

### Server (terobit_server)
Runs .server.tbpk files, and handles client connections for realm servers. Not started. See [README](bins/terobit_server/README.md).

## Editing
For those who edit this repo, please read [CONVENTIONS](CONVENTIONS.md) to read about rules and conventions set in place.

## License

This repository is **source-available, not open source**. You're welcome to read the code, but you may not use, copy, modify, or redistribute it for any purpose without explicit written permission. See [LICENSE](LICENSE) for details.

For permission requests, contact [contact@terobit.io](mailto:contact@terobit.io).
