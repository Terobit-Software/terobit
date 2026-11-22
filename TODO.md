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
| pack writing (`cargo xtask pkgen`) | works, writes unsigned v1 archives                                        |
| pack reading                 | stub: only checks a file is at least 24 bytes                             |
| dev directory loading        | works: assets load by handle through `pack://`, only the reading side's entries |
| signing / licenses           | signing exists in the writer, no key loading, no verification             |
| client                       | opens an empty window. `--dev` loads a dev directory, nothing uses it yet |
| server, editor, sandbox      | not started (hello world or empty)                                        |
| tests                        | 69 pass in a default workspace run (`runtime_assets` 45), `client_dev` runs 10 with `--features dev` |
| CI                           | `cargo fmt --check` failed on this branch (one import order), fixed 2026-10-24 |

## 1. Not built yet

Roughly in the order they depend on each other.

- [ ] **F1. Read `.tbpk` archives.**
  Follow "reading a pack" in [tbpk v1](docs/formats/tbpk/v1.md#reading-a-pack): common
  header (`HeaderReader::read_header` is an empty stub), table of contents and its limits,
  manifest and signature, index hash, then chunk reads that check the hash before
  decompressing.
  - Needs: `Deserialize` on every format type (they only derive `Serialize` now).
  - Test: a round trip, `cargo xtask pkgen` on `basic1`, read every file back, compare bytes. Then
    add the golden files [VERSIONING](docs/formats/common/VERSIONING.md#golden-files)
    describes.

- [ ] **F2. Signing keys and signature checks.**
  `PackSigner` can sign, but nothing loads a key (`xtask/src/pkgen.rs` has a TODO for it),
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

## 2. Cleanup

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

- [ ] **C7. Rewrite `cargo xtask regen`** (`xtask/src/regen.rs`). Its module doc says it
  needs a rewrite. It also has no size class between 4 KiB and 256 KiB, the sizes that
  fill shared small-file chunks, unless you use `--type random`.

- [ ] **C8. `basic1_should_fail` copies 8.7 MB of assets it never reads.** It only tests that
  `self:cube.material` is rejected, which fails before any file is opened. Keep only its
  `manifest.toml`, or move the case into an inline test like the other manifest tests.

- [ ] **C9. Make the reader plugin tests check something.** The `handles_*` tests in
  `readers/plugin.rs` only prove nothing panics. Test `PackReader::read_pack` directly and
  assert on the error variant.

- [ ] **C10. Remove or finish stubs.** `HeaderReader` (empty), `ManifestSource::get_v1`
  (never called), and the `// implemented` comments in `PackWriter`.

## 3. Tooling and CI

- [ ] **T1. Pin the Rust version.** `rust-toolchain.toml` lists targets but no `channel`, so
  every fresh machine downloads std for all five targets, and the compiler version depends
  on whatever `stable` is that day. Add `channel = "1.99.1"` (or the version you want) so
  local and CI builds match.

- [ ] **T2. Check macOS and Windows in CI.** CI only runs on Linux, but the toolchain targets
  macOS and Windows, and `runtime_logging` has code and tests that only compile on those.
  Add at least `cargo check` on `macos-latest` and `windows-latest`.

- [ ] **T3. Add clippy.** `cargo clippy --workspace --all-targets -- -D warnings` once C2 is
  done.

## 4. Docs to write

- [ ] Revocation list format ([INTEGRITY](docs/formats/common/INTEGRITY.md#revocation) says
  it gets its own doc).
- [ ] The signed "last logged in user" record (F3).
- [ ] The signed developer token that lets a released editor play a dev directory
  ([tbpk REF](docs/formats/tbpk/REF.md#dev-directory)).
- [ ] The `terobit_v1` host API for scripts (F6).

## Later

- Use cargo-packager for distribution. Running the binary directly is fine for now.
