# terobit manifest - .tbman

describes a pack: whether it is for the client or the server, which realm it belongs to,
who made it, who can run it, and every file in it with its size and hash. the manifest is the only file that gets signed, so it is
what makes every other file in the pack trustworthy (see
[integrity](../common/INTEGRITY.md)).

written by the pack tool from a human-written [`manifest.toml`](source.md). never edited by
hand.

current version: v1 (draft)
oldest supported: v1

## versions

- [v1](v1.md)
- [source format](source.md) (`manifest.toml`, not versioned on the wire)

## changelog

- v1: initial version. postcard body, blake3 hashes per file, `target` for separate client
  and server packs, scripts listed in the manifest (replaces the `.tbwas` wrapper)
