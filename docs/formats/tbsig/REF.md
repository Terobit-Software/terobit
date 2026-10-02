# terobit signature - .tbsig

proves a pack's `manifest.tbman` came from the platform and wasn't changed. the manifest
then covers every other byte of the pack through the index hash. see
[integrity](../common/INTEGRITY.md) for the whole chain of trust.

every published pack has exactly one, stored in the pack's signature section and next to
the manifest in a client's install. carries the certificate for the key that signed it,
so it can be checked offline. dev directories have none.

current version: v1 (draft)
oldest supported: v1

## versions

- [v1](v1.md)

## changelog

- v1: initial version. ed25519, signing key certified by a root key
