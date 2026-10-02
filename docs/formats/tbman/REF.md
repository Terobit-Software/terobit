# terobit manifest - .tbman

describes a pack: whether it's for the client or the server, which realm and version it
belongs to, whether playing needs a license, every file in it, and the hash of the pack's
index. the manifest is the only part of a pack that gets signed. through the index hash it
covers every byte of file data (see [integrity](../common/INTEGRITY.md)).

written by the pack tool from a human-written [`manifest.toml`](source.md). never edited by
hand. stored inside a [`.tbpk`](../tbpk/REF.md), and next to the index in a client's
install.

current version: v1 (draft)
oldest supported: v1

## versions

- [v1](v1.md)
- [source format](source.md) (`manifest.toml`, never sent anywhere so not versioned)

## changelog

- v1: initial version. postcard body, `target` for separate client and server packs,
  `access` for offline license checks, index hash instead of per-file hashes, scripts
  listed in the manifest (replaces the `.tbwas` wrapper), zstd pinned per manifest version
