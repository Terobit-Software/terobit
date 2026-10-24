# TODO

What's broken, what's missing, and what's out of date, with why each one matters and how
to do it. Items have ids (`B1`, `F3`, ...) so issues, PRs and other docs can point at them.
Tick an item or delete it in the PR that finishes it.

Last full review: 2026-10-24, on `feature/make_load_from_dev_work`. For how the pieces fit
together, see [ARCHITECTURE](docs/ARCHITECTURE.md).

## Where things stand

| area                         | state                                                                     |
|------------------------------|---------------------------------------------------------------------------|
| format docs (`docs/formats`) | detailed and mostly current, all versions still draft                     |
| manifest.toml loading        | works, validated, well tested                                             |
| pack writing (`cargo pkgen`) | works, writes unsigned v1 archives                                        |
| pack reading                 | stub: only checks a file is at least 24 bytes                             |
| dev directory loading        | parses and validates `manifest.toml`, then stops                          |
| asset registry (`pack://`)   | written but not connected to anything, see B1-B4                          |
| signing / licenses           | signing exists in the writer, no key loading, no verification             |
| client                       | opens an empty window, loads no game content                              |
| server, editor, sandbox      | not started (hello world or empty)                                        |
| tests                        | 70 passing in the non-graphics crates (`runtime_assets` 41, run flags 10, client_dev 10, others 9) |
| CI                           | `cargo fmt --check` failed on this branch (one import order), fixed 2026-10-24 |

## 1. Broken now

Fix these first. Each one stops something that looks finished from working.

- [ ] **B1. The asset registry isn't connected.**
  `PackRegistry` is never added as a resource, `PackAssetReader` is never registered as the
  `pack://` asset source (the compiler warns it is never constructed), and nothing ever
  adds a `TbpkSource` to the registry.
  - Why: any system that takes `PackAssets` won't run, because Bevy reports the registry
    resource as missing, and once that's fixed every `load` still fails. Loading assets from a dev directory, the goal of this
    branch, can't work until these three pieces connect.
  - How: create one `PackRegistry`, `insert_resource` a clone, and call
    `app.register_asset_source("pack", AssetSourceBuilder::new(move || Box::new(PackAssetReader(registry.clone()))))`.
    Bevy only accepts asset sources registered **before** `AssetPlugin` is added, and
    `RuntimeCorePlugins` adds `AssetPlugin` first (`runtime_core/src/lib.rs:29`), so the
    registration needs its own small plugin placed before `AssetPlugin` in that group.

- [ ] **B2. `PackAssets::load` and `PackAssetReader::read` disagree on what a path is.**
  `load` resolves the handle to a file path and asks for `pack://self/meshes/mesh.obj`
  (`registry/pack_assets.rs:33`). The reader then tries to parse that path as a handle
  like `self:cube` (`registry/pack_reader.rs:18`), which fails, so it returns not found.
  - Why: even with B1 fixed, no asset loads.
  - How: resolve once. Recommended: `load` resolves the handle and passes the pack path;
    the reader takes a pack path and only reads it. Keeping the real file name in the path
    also lets Bevy pick the right loader from the extension (`.glb`, `.png`, ...).

- [ ] **B3. The registry only looks up realm id `0`.**
  `resolve` and `read_bytes` both call `sources.get(&0u64)` (`registry/pack_registry.rs:63`,
  `:82`), but realm ids are never 0 and the manifest loader rejects 0.
  - Why: a source stored under its real `realm_id` can never be found.
  - How: while only one pack is loaded at a time, store the active realm id in the registry
    and use it for `self:` handles. Later, map namespaces to realms.

- [ ] **B4. Dev directory loading stops after reading the manifest.**
  `PackReader::read_pack` loads `manifest.toml` into `_manifest_source` and drops it
  (`readers/tbpk.rs:65`).
  - Why: `--dev --pack-file <dir>` logs success but nothing from the directory is usable.
  - How: build the client side's `ManifestV1` (`get_v1_side(Target::Client, ...)`) and add
    a `TbpkSource` with `RootLocation::Developer { path }` to the registry. The dev path
    has no index, so either make `IndexRef` optional for dev sources or give `TbpkSource`
    its own handle-to-path map built straight from `ManifestSource`.

- [ ] **B5. `TbpkSource::new` can panic.**
  It calls `unwrap()` on every id parse (`registry/tbpk_source.rs:19`, `:26`) and indexes
  `manifest.files[...]` without a bounds check. Assets and scripts also share one map, so
  an asset and a script with the same id (allowed by the tbman v1 rules) overwrite each
  other.
  - Why: a bad manifest crashes the client instead of failing the load with an error, which
    breaks the no-unwrap rule in [CONVENTIONS](CONVENTIONS.md).
  - How: return `Result<Self, TbpkSourceError>`, use `files.get(i)`, and keep separate maps
    for assets and scripts.

- [ ] **B6. The pack writer reads file sizes from disk twice.**
  `build_tbpk` gets the side's file list in `chunk()` (`pack/writers/tbpk.rs:318`) and
  again in `get_v1_side` (`readers/manifest_reader.rs:343`) when building the manifest.
  - Why: if a file changes size in between, the manifest's `size` won't match the chunked
    bytes and the pack is corrupt without any error.
  - How: call `get_side_files` once in `build_tbpk` and pass the list to both the chunker
    and the manifest builder.

- [ ] **B7. Some manifest rules only run when packing.**
  "id in both `[assets]` and `[client.assets]`" and "scripts must be `.wasm`" are checked in
  `get_v1_side` (`readers/manifest_reader.rs:333`, `:339`), not in `validate()`.
  - Why: the docs say the dev loader and the pack tool enforce the same rules, so mistakes
    show up during development. Right now a broken manifest loads fine with `--dev` and only
    fails at `cargo pkgen`.
  - How: move both checks into `ManifestSource::validate` and add tests for them there.

- [ ] **B8. `--dev` works in every build, including release.**
  The docs say `--dev` only exists in builds with a `dev` cargo feature and release
  binaries never have it ([tbpk dev directory](docs/formats/tbpk/REF.md#dev-directory)).
  No such feature exists, and `cargo rcl --final` still accepts `--dev`.
  - Why: a released client that reads unsigned dev directories skips signing and license
    checks entirely.
  - How: add `dev = []` to `client_dev` and a forwarding `dev = ["client_dev/dev"]` to
    `terobit_client`, wrap the `--dev` handling in `#[cfg(feature = "dev")]`, and have
    `cargo rcl` build with `--features dev` unless `--final` is passed.

- [ ] **B9. Every program's log file is named `terobit-client.log`.**
  The name is hard coded (`runtime_logging/src/lib.rs:110`), so the pack generator and run
  script also write `terobit-client.log` into their own log folders. The folder layout also
  differs per OS: `TerobitClient/` on macOS, `terobitclient/logs` on Linux,
  `terobit/client/logs` on Windows.
  - Why: confusing when looking for a tool's logs, and the per-OS layouts make support
    instructions harder.
  - How: build the file name from `first_name` and `last_name`, and pick one folder layout.

## 2. Docs and code disagree

The [format README](docs/formats/README.md) says that when code and docs disagree, one of
them is a bug.

- [ ] **D1. zstd level: docs say 19, code says 5.**
  [tbman v1](docs/formats/tbman/v1.md#compression) pins level 19. `ZSTD_LEVEL` is 5
  (`pack/types/tbman.rs:4`), and `cargo pkgen` uses it by default.
  - Why: the level changes every compressed chunk's bytes and hash, and it can never change
    once v1 is released.
  - How: decide before release. 19 gives smaller packs but packs much more slowly, 5 is
    quick. Update whichever side is wrong, and record the zstd version in tbman v1 at the
    same time (the doc still says "not pinned yet"). The zstd bump that had to be reverted
    in #21 shows why it should be pinned with `=` in `Cargo.toml` (see T4).

- [ ] **D2. The `writer` cargo feature does nothing.**
  `runtime_assets` declares `writer = [] # seperate with a feature`, but `pack::writers` is
  always compiled, so the client ships the pack writer.
  - How: put `pack::writers` behind `#[cfg(feature = "writer")]` (`tbpk_generator` already
    turns it on), or remove the feature.

- [x] **D3.** The README said `cargo rcl` runs the client with `--pack-file .`. It passes
  the absolute path of `tests/examples/source_to_tbpk/basic1`. Fixed in the README.
- [x] **D4.** The README listed `/rust-toolchain.md`. The file is `rust-toolchain.toml`.
  Fixed.
- [x] **D5.** [tbpk REF](docs/formats/tbpk/REF.md#building) said the pack tool in
  `runtime_assets::pack` was planned. It exists at `runtime_assets::pack::writers`. Fixed.

## 3. Not built yet

Roughly in the order they depend on each other.

- [ ] **F1. Read `.tbpk` archives.**
  Follow "reading a pack" in [tbpk v1](docs/formats/tbpk/v1.md#reading-a-pack): common
  header (`HeaderReader::read_header` is an empty stub), table of contents and its limits,
  manifest and signature, index hash, then chunk reads that check the hash before
  decompressing.
  - Needs: `Deserialize` on every format type (they only derive `Serialize` now).
  - Test: a round trip, `cargo pkgen` on `basic1`, read every file back, compare bytes. Then
    add the golden files [VERSIONING](docs/formats/common/VERSIONING.md#golden-files)
    describes.

- [ ] **F2. Signing keys and signature checks.**
  `PackSigner` can sign, but nothing loads a key (`tbpk_generator` has a TODO for it),
  nothing verifies a `.tbsig`, no root public keys are built in, and the revocation list
  has no format yet.

- [ ] **F3. Licenses (`.tblic`).**
  `LicenseV1` is a bare struct: no serde, no encode or decode, no check. The signed "last
  logged in user" record it depends on isn't defined anywhere
  ([tblic REF](docs/formats/tblic/REF.md#what-this-can-and-cant-stop)).

- [ ] **F4. Load packs without `--dev`.**
  Without `--dev` the client ignores `--pack-file` and loads nothing. Needs F1, and F2 for
  signed packs.

- [ ] **F5. Server.** `terobit_server` prints "Hello, world!". See its
  [README](bins/terobit_server/README.md) for the plan.

- [ ] **F6. Script sandbox.** `runtime_sandbox` is an empty file and nothing depends on it.
  Scripts are plain `.wasm` that may only import versioned `terobit_v1`, `terobit_v2`, ...
  host modules, with no WASI ([tbman v1 scripts](docs/formats/tbman/v1.md#scripts)). Pick a
  wasm runtime and define `terobit_v1`.

- [ ] **F7. Editor.** `terobit_editor` prints "Hello, world!". See its
  [README](bins/terobit_editor/README.md).

- [ ] **F8. Async, positional reads in the asset reader.** `PackRegistry::read_bytes` is a
  blocking `std::fs::read` inside Bevy's async reader (`registry/pack_registry.rs:95`, the
  code notes it). Use positional reads, never memory maps, as the
  [installed form](docs/formats/tbpk/REF.md#installed) explains.

- [ ] **F9. Folder loading.** `read_directory` and `is_directory` in `PackAssetReader`
  always return not found. Answer them from the manifest's file list.

- [ ] **F10. Configurable chunk size.** `PackWriter` always uses 1 MiB and has no setter.
  `MIN_CHUNK_SIZE` and `MAX_CHUNK_SIZE` are unused until the reader checks them (F1).

- [ ] **F11. Installed store.** The client's on-disk store (chunks by hash, partial updates)
  from [tbpk REF](docs/formats/tbpk/REF.md#installed).

## 4. Cleanup

- [ ] **C1. The chunker logs at `info!` for every file and every chunk.** 29 `info!` calls
  in `chunker.rs`, several per chunk. Packing the 4 files in `basic1` prints 365 lines, so
  a real game prints hundreds of thousands. Log levels are also fixed at INFO
  (`runtime_logging/src/lib.rs:128`, `:139`).
  - How: move per-file and per-chunk lines to `debug!` or `trace!`, keep the summaries at
    `info!`, add an `EnvFilter` so `RUST_LOG` works, and allow `debug!`/`trace!` in
    CONVENTIONS.

- [ ] **C2. Fix the 16 compiler warnings in `runtime_assets`.** Mostly unused imports in
  `pack/writers/tbpk.rs`, plus `PackWriter::header`'s unused `target`, the unused
  `commands` in `read_pack`, and never-used methods like `ManifestSource::get_v1`.
  `cargo fix --lib -p runtime_assets` handles 11 of them.

- [ ] **C3. Encode the index once per build.** `encode_index` runs three times per pack
  (manifest, table of contents, write).

- [ ] **C4. Error names and messages.** `HandleError::UknownHandle` is misspelled, never
  used, and its message ends with a period (CONVENTIONS says no trailing period).
  `RegistryError` variants all start with `Failed` and say "realmid". `InvalidHandle` says
  "asset id" when the bad id is in a scripts table.

- [ ] **C5. `ClientDevPlugin` panics if run flags are missing.** It calls
  `resource::<RuntimeFlags>()` (`client_dev/src/lib.rs:24`). Use `get_resource` and warn,
  like it already does for `RuntimePackConfig`.

- [ ] **C6. One argument parser.** `run_client_script` uses clap. `tbpk_generator` and
  `example_generator` use `RuntimeFlags` and validate by hand, so they have no `--help`.
  clap is already a workspace dependency.

- [ ] **C7. Rewrite `example_generator`.** Its first line says it needs a rewrite. It also
  has no size class between 4 KiB and 256 KiB, the sizes that fill shared small-file
  chunks, unless you use `--type random`.

- [ ] **C8. `basic1_should_fail` copies 8.7 MB of assets it never reads.** It only tests that
  `self:cube.material` is rejected, which fails before any file is opened. Keep only its
  `manifest.toml`, or move the case into an inline test like the other manifest tests.

- [ ] **C9. Make the reader plugin tests check something.** The `handles_*` tests in
  `readers/plugin.rs` only prove nothing panics. Test `PackReader::read_pack` directly and
  assert on the error variant.

- [ ] **C10. Remove or finish stubs.** `HeaderReader` (empty), `ManifestSource::get_v1`
  (never called), and the `// implemented` comments in `PackWriter`.

## 5. Tooling and CI

- [ ] **T1. Pin the Rust version.** `rust-toolchain.toml` lists targets but no `channel`, so
  every fresh machine downloads std for all five targets, and the compiler version depends
  on whatever `stable` is that day. Add `channel = "1.99.1"` (or the version you want) so
  local and CI builds match.

- [ ] **T2. Check macOS and Windows in CI.** CI only runs on Linux, but the toolchain targets
  macOS and Windows, and `runtime_logging` has code and tests that only compile on those.
  Add at least `cargo check` on `macos-latest` and `windows-latest`.

- [ ] **T3. Add clippy.** `cargo clippy --workspace --all-targets -- -D warnings` once C2 is
  done.

- [ ] **T4. Keep dependabot away from zstd once v1 is pinned.** Daily grouped updates
  include zstd, and a new zstd can change chunk bytes. Add an `ignore` entry for it.

## 6. Docs to write

- [x] READMEs for every binary and script (were `# TODO`). Done, update them as things change.
- [x] [ARCHITECTURE](docs/ARCHITECTURE.md): how the pieces fit together and why.
- [x] [runtime_assets README](crates/runtime/runtime_assets/README.md).
- [ ] Revocation list format ([INTEGRITY](docs/formats/common/INTEGRITY.md#revocation) says
  it gets its own doc).
- [ ] The signed "last logged in user" record (F3).
- [ ] The signed developer token that lets a released editor play a dev directory
  ([tbpk REF](docs/formats/tbpk/REF.md#dev-directory)).
- [ ] The `terobit_v1` host API for scripts (F6).

## Later

- Use cargo-packager for distribution. Running the binary directly is fine for now.
