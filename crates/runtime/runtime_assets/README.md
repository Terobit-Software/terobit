# runtime_assets

Reads and writes Terobit packs, loads `manifest.toml` dev directories, and will serve pack
files to Bevy's asset server. The format docs in [docs/formats](../../../docs/formats/README.md)
are the spec this crate follows.

## Modules

| module          | what's in it                                                                 | state        |
|-----------------|------------------------------------------------------------------------------|--------------|
| `pack` (types)  | in-memory form of each format: `CommonHeader`, `TbPack` and `IndexV1`, `ManifestV1`, `TbSignature`, `TbLicense`, `Certificate` and the hash contexts, `AssetHandle`, `validate_path` | serialize only, no `Deserialize` yet (F1) |
| `pack::writers` | `PackWriter` builds and writes `.tbpk` archives, `PackSigner` signs manifests, the chunker splits and compresses files | working |
| `readers`       | `ManifestSource` loads and validates `manifest.toml`. `RuntimePackReaderPlugin`, `LoadPackFileEvent` and `RuntimePackConfig` start a pack load. `PackReader` reads it | manifest and dev directories work, `.tbpk` reading is a stub (F1) |
| `registry`      | `PackRegistry` holds the loaded pack, `PackAssets` loads by handle in systems, `PackAssetReader` is the `pack://` asset source | working for dev directories |

Ids like `B1` and `F1` are items in the [TODO](../../../TODO.md).

## Using it

### Build packs

```rust
use runtime_assets::pack::writers::PackWriter;
use runtime_assets::pack::{Target, ZSTD_LEVEL};

let mut writer = PackWriter::new().with_zstd_level(Some(ZSTD_LEVEL))?;
writer.load_manifest("tests/examples/source_to_tbpk/basic1/manifest.toml".into())?;

for (target, name) in [(Target::Client, "client"), (Target::Server, "server")] {
    writer.build_tbpk(target)?;
    writer.write_tbpk_out(format!("out/1-1.{name}.tbpk").as_ref())?;
}
```

Build and write one target before building the next. The writer only keeps the most
recently built pack. Add `.with_signer(PackSigner::new(key, cert)?)` to sign. Without it
the pack has an empty signature section. `cargo pkgen` wraps all of this.

### Start loading a pack

`RuntimePackReaderPlugin` is already part of `RuntimeCorePlugins`. In your own app, add it
before `AssetPlugin`. To load a pack, trigger the event with the side that reads it:

```rust
world.trigger(LoadPackFileEvent { path: path.into_boxed_path(), target: Target::Client });
```

Directories (dev directories) are only read when `RuntimePackConfig::set_allow_dir_read(true)`
has been called, which `client_dev` does for `--dev`.

### Load an asset by handle

Works for dev directories. `.tbpk` archives come with F1.

```rust
fn spawn(assets: PackAssets) {
    let handle = "self:cube".parse().expect("valid handle");
    let mesh: Handle<Mesh> = assets.load(&handle).expect("declared in the manifest");
}
```

## Rules this crate enforces

- paths: 1-16 segments, `a-z 0-9 . _ -`, no leading or trailing dot, no Windows reserved
  names, at most 255 bytes (`validate_path`)
- handles: `self:<name>`, name uses `a-z 0-9 _ - /` (`AssetHandle`)
- manifests: `realm_id` and `owner_id` aren't 0, unknown fields are rejected, every id and
  path is checked on load, no id is in both `[assets]` and a side's `assets`, and scripts
  are `.wasm`
- reading: only the shared entries and the reading side's are visible, by handle or by
  `pack://` path
- writing: files sorted by path, chunks laid out as in
  [writing chunks](../../../docs/formats/tbpk/v1.md#writing-chunks), a chunk that zstd
  shrinks by less than 20% is stored raw, and a source file that changes size while
  packing is an error

## Tests

```sh
cargo test -p runtime_assets
```

45 tests cover paths, handles, manifest loading (against `tests/examples/source_to_tbpk`),
the chunker (layout, round trips, determinism, files changing size), the reader plugin, and
loading `basic1` assets end to end through `pack://`.
Nothing tests a whole written pack yet. That comes with the reader (F1).
