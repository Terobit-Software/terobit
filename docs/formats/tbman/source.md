# manifest source - manifest.toml

the human-written manifest that lives in a dev directory. you write this, the pack tool
compiles it into **two** [`manifest.tbman`](v1.md) files, one for the client pack and one
for the server pack. it has no sizes, hashes or indexes, since the tool works those out.

with `--dev` (only in builds with the `dev` feature), the client and server read this
directly from the dev directory, so you can edit assets and scripts without rebuilding
anything. each side only reads its own sections, the same as it would from a built pack.
see
[tbpk dev directories](../tbpk/REF.md#dev-directory).

## example

```toml
realm_id = 1
owner_id = 1
pack_version = 1
access = "licensed"     # or "free"

# assets both sides need, copied into both packs
[assets]
"self:map" = "models/map.glb"

# client only: never put in the server pack
[client.assets]
"self:grass" = "textures/grass.png"
"self:music" = "audio/theme.ogg"

[client.scripts]
"self:main" = "scripts/client.wasm"

# server only: never put in the client pack
[server.assets]
"self:loot_tables" = "data/loot.bin"

[server.scripts]
"self:main" = "scripts/server.wasm"
```

## fields

| key                | required | goes into                                    |
|--------------------|----------|----------------------------------------------|
| `realm_id`         | yes      | both packs                                   |
| `owner_id`         | yes      | both packs                                   |
| `pack_version`     | yes      | both packs                                   |
| `access`           | yes      | both packs, `"free"` or `"licensed"`         |
| `[assets]`         | no       | `assets` of both packs, id = path            |
| `[client.assets]`  | no       | `assets` of the client pack only             |
| `[client.scripts]` | no       | `scripts` of the client pack only, id = path |
| `[server.assets]`  | no       | `assets` of the server pack only             |
| `[server.scripts]` | no       | `scripts` of the server pack only, id = path |

an id can't appear in both `[assets]` and a side's `[*.assets]`. the client and server
namespaces are separate, so `self:main` can be a script on both sides.

each pack's `files` is built from every path referenced by that pack's entries,
deduplicated and sorted. a file that nothing references isn't packed. every path must
follow the [path rules](../common/CONVENTIONS.md#paths-inside-a-pack), checked by both the
dev loader and the pack tool.

code shared by both sides is shared at the source level (a rust crate both scripts depend
on) and compiled into each side's `.wasm` separately.

## not versioned

this file never goes over the wire, so it has no version header. when the manifest format
changes, the pack tool learns the new fields and old `manifest.toml` files keep working
with defaults where possible.
