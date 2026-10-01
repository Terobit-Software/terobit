# integrity

how terobit knows a pack hasn't been tampered with, and who it came from.

two separate pieces:

- **hashes** prove each file matches the manifest
- **the signature** ([`.tbsig`](../tbsig/REF.md)) proves the manifest itself came from
  someone trusted

together the signature covers the manifest and the manifest covers every file, so one
signature check protects the whole pack.

## hashes

[blake3](https://github.com/BLAKE3-team/BLAKE3), 32 bytes, over the raw uncompressed file
bytes. blake3 is faster than sha-256 and can hash in parallel, so large assets stay cheap.

files are verified **when they are loaded**, not all up front. a 2 gb game doesn't hash 2 gb
before the first frame, only what it actually loads. a file whose hash doesn't match fails
to load and the error is logged with the path.

hashes are never written by hand. the pack tool computes them when it builds a pack (see
[tbman source](../tbman/source.md)).

## signing

ed25519. there is no encryption. the client has to read a pack to run it, so any key that
could decrypt it would ship inside the client and protect nothing. signing is what stops a
modified pack from running.

server code stays private because it is never sent to players at all, not because it is
encrypted (see [tbpk](../tbpk/REF.md#client-and-server-packs)).

### what gets signed

the exact bytes of `manifest.tbman`, header included. nothing is re-encoded before checking,
so there is no way for two encoders to disagree. layout and verify steps are in
[tbsig v1](../tbsig/v1.md).

### who signs

| pack                     | signed by     | behavior                                       |
|--------------------------|---------------|------------------------------------------------|
| client pack from terobit | platform key  | client must verify, refuse to run otherwise    |
| server pack from terobit | platform key  | server must verify, refuse to run otherwise    |
| local dev directory      | nobody        | only loads with the dev flag                   |

the platform signs both packs when a realm is published, after server side checks pass.
the manifest's `target` field is inside the signature, so a client pack can't be passed
off as a server pack or the other way round.

developer keys (so a dev's own builds are recognized as theirs) can be added later as a new
`key_kind` in a new tbsig version.

### key rotation

the client and server ship with a list of trusted platform public keys. adding a key is an
update. old keys are removed once nothing signed with them is still published.
