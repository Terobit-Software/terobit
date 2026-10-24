# runtime_assets

Reads and writes Terobit packs, loads `manifest.toml` dev directories, and will serve pack
files to Bevy's asset server. The format docs in [docs/formats](../../../docs/formats/README.md)
are the spec this crate follows.

## Modules

| module          | what's in it                                                                 | state        |
|-----------------|------------------------------------------------------------------------------|--------------|
| `pack` (types)  | in-memory form of each format: `CommonHeader`, `TbPack` and `IndexV1`, `ManifestV1`, `TbSignature`, `TbLicense`, `Certificate` and the hash contexts, `AssetHandle`, `validate_path` | serialize only, no `Deserialize` yet (F1) |
| `pack::writers` | `PackWriter` builds and writes `.tbpk` archives, `PackSigner` signs manifests, the chunker splits and compresses files | working |
| `readers`       | `ManifestSource` loads and validates `manifest.toml`. `RuntimePackReaderPlugin`, `LoadPackFileEvent` and `RuntimePackConfig` start a pack load. `PackReader` reads it | manifest works, pack reading is a stub (F1, B4) |
| `registry`      | `PackRegistry` maps handles to files, `PackAssets` loads by handle in systems, `PackAssetReader` is the `pack://` asset source | written, not connected (B1-B5) |

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

`RuntimePackReaderPlugin` is already part of `RuntimeCorePlugins`. To load a pack, trigger
the event:

```rust
world.trigger(LoadPackFileEvent { path: path.into_boxed_path() });
```

Directories (dev directories) are only read when `RuntimePackConfig::set_allow_dir_read(true)`
has been called, which `client_dev` does for `--dev`.

### Load an asset by handle

This is the intended API. It doesn't work yet (B1-B4).

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
  path is checked on load. Two more checks only run at pack time for now (B7)
- writing: files sorted by path, chunks laid out as in
  [writing chunks](../../../docs/formats/tbpk/v1.md#writing-chunks), a chunk that zstd
  shrinks by less than 20% is stored raw, and a source file that changes size while
  packing is an error

## Tests

```sh
cargo test -p runtime_assets
```

41 tests cover paths, handles, manifest loading (against `tests/examples/source_to_tbpk`),
the chunker (layout, round trips, determinism, files changing size) and the reader plugin.
Nothing tests a whole written pack yet. That comes with the reader (F1).
