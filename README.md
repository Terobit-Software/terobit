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
