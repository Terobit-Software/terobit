# Terobit
The engine repo for the Terobit project. This does NOT contain the website. That is hosted on a seperate repo for security reasons.

## Commands

### cargo rcl
Build and run the client. builds `terobit_client`, copies it and `bins/terobit_client/assets` into a fresh `out/client`, and runs it from there with `--dev --pack-file .`.

|args|desc|
|----|----|
|--final, -f|builds with the release profile instead of release-dev, and runs without the dev flags|
|--strip-defaults, -s|runs without the default dev flags (`--dev`, `--pack-file .`)|

### cargo pkgen
Build .tbpk packs from a manifest.toml. packs are named `<realm_id>-<pack_version>.<client\|server>.tbpk`.

|args|desc|
|--|--|
|--manifest|path to the manifest.toml, required. asset paths are relative to its folder|
|--out|folder the packs are written to, created if missing. defaults to `out/tbpk_generator`|
|--target|client, server, or both. defaults to both|
|--zstd-level|zstd level chunks are compressed at, or `none` to store them uncompressed. defaults to `ZSTD_LEVEL`, any other level changes chunk bytes and hashes|

### cargo regen
Generate example dev directories in `tests/examples/source_to_tbpk/generated`, for test and example packs. existing examples are skipped.

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

</details>

## Binaries

### Pack Generator (tbpk_generator)
Generates .tbpk files from a manifest and asset directory. See [README](bins/tbpk_generator/README.md).

### Client (terobit_client)
Runs and parses .client.tbpk files. See [README](bins/terobit_client/README.md).

### Editor (terobit_editor)
Runs, parses, edits, and creates .tbpk files. The main editing suite for Terobit realms. Current replacement is tbpk_generator due to the primitive state of the project. See [README](bins/terobit_editor/README.md).

### Server (terobit_server)
Runs .server.tbpk files, and handles client connections for realm servers. Currently does nothing due to the primitive state of the project. Runs and parses .tbpk files. See [README](bins/terobit_server/README.md).

## Editing
For those who edit this repo, please read [CONVENTIONS](CONVENTIONS.md) to read about rules and conventions set in place.

## License

This repository is **source-available, not open source**. You're welcome to read the code, but you may not use, copy, modify, or redistribute it for any purpose without explicit written permission. See [LICENSE](LICENSE) for details.

For permission requests, contact [contact@terobit.io](mailto:contact@terobit.io).
