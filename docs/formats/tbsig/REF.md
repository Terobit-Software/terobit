# terobit signature - .tbsig

an ed25519 signature over a pack's `manifest.tbman`. it proves the manifest came from the
platform and wasn't changed, and the manifest's hashes then cover every other file in the
pack. see [integrity](../common/INTEGRITY.md) for how signing fits together.

every archive pack has exactly one, at `manifest.tbsig`. dev directories have none.

current version: v1 (draft)
oldest supported: v1

## versions

- [v1](v1.md)

## changelog

- v1: initial version. ed25519, platform keys only
