# Architecture

How the pieces of this repo fit together, why they're built this way, and how far along
each one is. Exact file layouts live in [formats](formats/README.md). Open work lives in
[TODO](../TODO.md), and items below link to it by id (`B1`, `F1`, ...).

## The big picture

A **realm** is one game. A developer writes it as a **dev directory**: a `manifest.toml`
plus the files it lists. Publishing turns it into two **packs**, one for players and one
for whoever hosts the game.

```
dev directory (manifest.toml + files)
   |
   |  cargo pkgen (tbpk_generator)                          working
   v
<realm_id>-<pack_version>.client.tbpk    .server.tbpk       unsigned
   |
   |  platform checks and signs (not in this repo)          not built
   v
terobit_client loads the client pack                        F1, F4
terobit_server loads the server pack                        F5
```

During development the client skips packing and reads the dev directory directly with
`--dev --pack-file <dir>`. `--dev` only exists in builds with the `dev` cargo feature, so a
release build can't read a dev directory.

## Binaries

| binary                                          | what it is                                     | state              |
|-------------------------------------------------|------------------------------------------------|--------------------|
| [terobit_client](../bins/terobit_client/README.md) | the game client players run                  | empty window       |
| [terobit_server](../bins/terobit_server/README.md) | runs server packs, hosts players             | hello world        |
| [terobit_editor](../bins/terobit_editor/README.md) | client plus tools, replaces `--dev` and pkgen | hello world       |
| [tbpk_generator](../bins/tbpk_generator/README.md) | `cargo pkgen`, builds packs                  | working            |
| [run_client_script](../scripts/run_client_script/README.md) | `cargo rcl`, builds and runs the client | working        |
| [example_generator](../scripts/example_generator/README.md) | `cargo regen`, makes test dev directories | working       |

## Crates

`crates/runtime` holds crates any binary can use. `crates/client` holds crates only the
client uses (`crates/server` and `crates/editor` will follow the same pattern).

| crate               | what it does                                                                 |
|---------------------|------------------------------------------------------------------------------|
| `runtime_core`      | base plugins every app needs: task pools, time, transforms, states, the asset server, run flags and the pack reader. Add it first |
| `runtime_assets`    | everything about packs: format types, `manifest.toml` loading, the pack writer, the pack reader and the asset registry. See its [README](../crates/runtime/runtime_assets/README.md) |
| `runtime_run_flags` | turns `--flag value value --other` into the `RuntimeFlags` resource         |
| `runtime_logging`   | terminal and daily log file output, plus a panic hook that logs panics       |
| `runtime_window`    | window, winit, clipboard, accessibility                                      |
| `runtime_render`    | renderer, meshes, cameras, lights, PBR, sprites, glTF, gizmos                |
| `runtime_ui`        | text, UI and UI widgets                                                      |
| `runtime_input`     | keyboard, mouse, gamepads, input focus                                       |
| `runtime_scene`     | scenes and world serialization                                               |
| `runtime_animation` | animation                                                                    |
| `runtime_audio`     | audio (vorbis)                                                               |
| `runtime_picking`   | Bevy's default picking                                                       |
| `runtime_sandbox`   | empty, will host wasm scripts (F6)                                           |
| `client_dev`        | the client's `--dev` and `--pack-file` handling, compiled only with its `dev` feature |

### Why so many small crates

Bevy is a workspace dependency with `default-features = false`, and each crate turns on
only the Bevy features it uses. A binary picks the crates it needs, so the pack generator
and the future server never compile the renderer or windowing. Small crates also rebuild
faster.

### Plugin order

The client adds plugin groups in this order: core, input, scene, window, render, ui,
animation, audio, picking, then client plugins.

- core goes first because window, render and scene all need the `AssetServer`
- render goes after window, and ui after render
- client plugins go last because they read resources the runtime adds (`RuntimeFlags`,
  `RuntimePackConfig`)
- inside core, `RuntimePackReaderPlugin` comes before `AssetPlugin`, because Bevy only
  accepts the `pack://` asset source it registers before `AssetPlugin` is added

## Building a pack

This part works. `PackWriter` in `runtime_assets::pack::writers` does it, and
`tbpk_generator` drives it.

1. `load_manifest` parses `manifest.toml` and checks ids, handles and every path against
   the [path rules](formats/common/CONVENTIONS.md#paths-inside-a-pack)
2. `build_tbpk(target)`, once per side:
   1. collect that side's files: shared `[assets]` plus `[client.*]` or `[server.*]`,
      sorted by path, with sizes read from disk
   2. chunk them (see [writing chunks](formats/tbpk/v1.md#writing-chunks)). Chunk bytes go
      to a temp spool file, so memory stays around one chunk however big the pack is
   3. encode the index and hash it
   4. build and encode the manifest, which holds the index hash
   5. sign the manifest bytes if a `PackSigner` was given (none is yet, F2)
   6. lay out the table of contents
3. `write_tbpk_out(path)` writes the header, table of contents, manifest, signature and
   index, then copies the data section from the spool file

## Loading a pack

What happens today when you run `terobit_client --dev --pack-file <dir>`, built with
`--features dev`:

1. `RuntimeRunFlagsPlugin` stores the command line in `RuntimeFlags`
2. `ClientDevPlugin` sees `--dev`, allows directory reads in `RuntimePackConfig`, and
   triggers `LoadPackFileEvent` with the path
3. `RuntimePackReaderPlugin`'s observer calls `PackReader::read_pack`
   - a directory: parses and validates `manifest.toml`, then puts the shared and
     `[client.*]` entries in `PackRegistry`
   - a file: checks it is at least 24 bytes, then stops (F1)
4. game code calls `PackAssets::load::<T>(&"self:cube".parse()?)`, which looks the handle up
   in the registry and loads `pack://meshes/mesh.obj`. The `pack://` reader only serves
   files the loaded side declares, so a client can't read server files even in dev

What reading a `.tbpk` should do (from the [tbpk docs](formats/tbpk/v1.md#reading-a-pack)):

1. read and check the header, table of contents, manifest and signature
2. check the license if the realm is `Licensed` (F3)
3. check the index hash and decode the index
4. add the pack to `PackRegistry`, then load assets the same way as from a dev directory.
   Each chunk is hash checked before it is decompressed

## Why it's built this way

Short versions. The format docs have the full reasoning.

- **Chunked packs with an index.** Reading one file never reads the whole pack, memory is
  bounded by the chunk size, and an update only downloads chunks that changed.
  ([tbpk REF](formats/tbpk/REF.md))
- **Chunks hashed as stored.** The hash is checked before zstd sees the bytes, so a
  malicious pack can't feed crafted input to the decompressor.
  ([integrity](formats/common/INTEGRITY.md#hashing))
- **postcard bodies, fixed width header and table of contents.** postcard is compact and
  needs no hand-written parser. The fixed parts let a reader seek before decoding
  anything. ([conventions](formats/common/CONVENTIONS.md#body-encoding))
- **Separate client and server packs.** Server assets and code never reach players,
  unless the realm allows p2p or offline play.
  ([tbpk REF](formats/tbpk/REF.md#client-and-server-packs))
- **Everything checks offline, nothing carries a date.** An offline machine's clock can't
  be trusted. Revocation, not expiry, stops a leaked key.
  ([integrity](formats/common/INTEGRITY.md#no-expiry-dates))
- **Strict path rules.** Every path means the same file on Windows, macOS and Linux, and no
  path can escape the pack. Loosening the rules later breaks nothing, tightening would.
  ([conventions](formats/common/CONVENTIONS.md#paths-inside-a-pack))
- **One implementation of the format.** The writer and reader live in the same crate, so
  the format is only implemented once. ([tbpk REF](formats/tbpk/REF.md#building))
