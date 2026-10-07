# terobit pack - .tbpk

everything one side needs to run one version of a realm: the manifest, its signature, an
index, and the file data. file data is split into chunks that can each be read and checked
on their own, so:

- reading one file never means reading the whole pack
- reading part of a big file only touches the chunks that cover that part
- memory use is bounded by the chunk size, however big the pack is

current version: v1 (draft)
oldest supported: v1

## versions

- [v1](v1.md)

## client and server packs

every publish builds two packs from one dev directory. the manifest's signed `target`
tells them apart.

| pack   | contains                                  | who gets it                                                                 |
|--------|-------------------------------------------|-----------------------------------------------------------------------------|
| client | shared + client assets and scripts        | players who can play the realm (it's free, or they own it)                  |
| server | shared + server assets and scripts        | whoever hosts it: official servers (one docker image per realm), developers, community hosts, and players for p2p and offline play |

server assets and scripts are never in a client pack. if a realm allows p2p or offline
play though, players get its server pack so they can host, and its server code isn't
secret from them.

suggested names: `<realm_id>-<pack_version>.client.tbpk` and
`<realm_id>-<pack_version>.server.tbpk`. readers trust the manifest's `target`, never the
filename.

## forms

a pack exists in three forms. only the archive is a stable, versioned format.

### archive

a single `.tbpk` file, laid out as in [v1](v1.md). used to upload a realm to the platform,
move a game between machines offline, and share with p2p peers.

the pack tool writes archives unsigned (empty signature section). the platform adds the
signature when it publishes. everything except the platform's upload service rejects
unsigned archives.

### installed

how a client keeps games on disk. this is internal to the client and **not** a stable
format: it can change between client versions without a format version bump.

```
store/
  chunks/<first 2 hex chars>/<hash as hex>       one file per chunk, stored bytes as-is
  packs/<realm_id>/<target>/<pack_version>/
    manifest.tbman
    manifest.tbsig
    index
```

- chunks are keyed by hash, so a chunk shared between versions or realms is stored once
- an update downloads the new manifest, signature and index, then only the chunks the
  store doesn't already have. a small patch to a 20 gb game downloads megabytes, not 20 gb
- the old version is deleted only after the new one is complete, then chunks nothing
  references are removed. an interrupted update leaves the old version playable
- chunks are hashed again on every read (blake3 is fast enough), which also catches disk
  corruption and local tampering
- files are read with positional reads, never memory mapped. on windows a mapped file
  can't be replaced while the game runs, and on macos and linux a mapped file that shrinks
  crashes the process
- an install can be exported to an archive, and an archive imported into the store, for
  moving games offline

### dev directory

a plain directory, used while building a game. it holds
[`manifest.toml`](../tbman/source.md) and the files it references, for both sides. no
manifest.tbman, no index, no signature.

dev directories load in two places:

- **now:** the client and server with `--dev`. the flag only exists in builds compiled with
  the `dev` cargo feature. release binaries are never built with it, so no released client
  or server can load a dev directory
- **later:** the editor, which is the client plus tools and replaces `--dev`. once it exists,
  the `dev` feature stays in the code but unused, in case it's needed again. a released
  editor should only play a dev directory for a developer of that `realm_id` (checked
  offline with a signed token), so a shared dev directory can't be used to pirate a realm

`--dev` is for local testing only:

- never logs into an account
- never connects to any remote server or to other players. the dev client only connects to
  a dev server on the same machine
- the dev client loads only shared and `[client.*]` entries, the dev server only shared and
  `[server.*]` entries, so dev behaves like production
- hashes, signatures and licenses aren't checked, so editing a file needs no rebuild
- path rules are enforced the same as in built packs

## building

the pack tool turns a dev directory into two archives:

1. read `manifest.toml` and check every path
2. for each target, collect that target's entries and files
3. split files into chunks and compress them (see [v1](v1.md#writing-chunks))
4. write the index, hash it, and write the manifest with that hash
5. write the archive with an empty signature section

the tool lives in `runtime_assets::pack` (planned), so the same code that reads packs also writes
them and there is only one implementation of the format to keep correct.

## changelog

- v1: own chunked format with an index. replaces the renamed `.tar.gz`
