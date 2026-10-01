# terobit pack - .tbpk

the container a game ships in. holds the manifest, its signature, and every file the
manifest lists. it comes in two forms: an **archive** for anything distributed, and a
**dev directory** for local development.

the pack itself is unversioned because it is a standard archive. everything that needs a
version (the manifest, the signature) carries its own [header](../common/HEADER.md).

## client and server packs

every publish produces two archive packs from the same dev directory:

| pack        | `target` | contains                                 | who gets it              |
|-------------|----------|------------------------------------------|--------------------------|
| client pack | `Client` | shared assets, client assets and scripts | players                  |
| server pack | `Server` | shared assets, server assets and scripts | game servers only, never players |

server scripts and server assets are never in a client pack in any form, so nothing a
player downloads can leak server logic. the manifest's `target` field (covered by the
signature) is what a reader trusts, not the filename.

suggested filenames: `<realm_id>-<pack_version>.client.tbpk` and
`<realm_id>-<pack_version>.server.tbpk`.

## contents

```
manifest.tbman      required, see tbman
manifest.tbsig      required in archives, see tbsig
<files>             every file listed in the manifest, at its listed path
```

nothing else is allowed. a file in the pack that the manifest doesn't list is an error, so
a pack can't smuggle in data that skipped hashing.

## archive form

a `tar` archive compressed with [zstd](https://facebook.github.io/zstd/), renamed to
`.tbpk`. zstd decompresses several times faster than gzip at a similar size, which matters
for load times.

- `manifest.tbman` must be the first entry and `manifest.tbsig` the second, so the reader
  can verify the signature and `target` before extracting anything else
- only regular files and directories. symlinks, hardlinks and devices are rejected
- every path follows the [path rules](../common/CONVENTIONS.md#paths-inside-a-pack)
- extracted size is checked against the manifest's file sizes while extracting, so a
  compression bomb stops at the first file that goes over

## dev directory form

a plain directory, used while building a game. it holds `manifest.toml` (the
[human-written source](../tbman/source.md)) instead of `manifest.tbman`, and no signature.
one dev directory holds both sides.

- the dev client only loads the shared and `[client.*]` entries, the dev server only the
  shared and `[server.*]` entries, so dev behaves like production
- hashes are not checked, so editing an asset needs no rebuild
- only loads when started with the dev flag
- never accepted for online play

## building

the pack tool turns a dev directory into both archives:

1. read `manifest.toml`
2. check every listed file exists and every path is valid
3. for each target, collect that target's entries
4. hash every file, record sizes
5. write each target's `manifest.tbman`
6. write each archive in the order above

the tool lives in `runtime_pack` (planned) so the same code that reads packs also writes
them. there is only one implementation of the format to keep correct. signing happens on
the platform when a realm is published, not in this tool.
