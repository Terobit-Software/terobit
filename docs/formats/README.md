# terobit file formats

reference for every custom file terobit reads or writes. the client, server, editor and
platform all follow these docs, so the docs are the source of truth. if code and docs
disagree, one of them is a bug.

## formats

| ext      | magic  | what it is                                                    | current | status |
|----------|--------|---------------------------------------------------------------|---------|--------|
| `.tbpk`  | `TBPK` | pack: everything one side (client or server) needs to run a realm | v1  | draft  |
| `.tbman` | `TBMN` | manifest: what is in a pack and who it belongs to             | v1      | draft  |
| `.tbsig` | `TBSG` | signature: proves a manifest came from the platform           | v1      | draft  |
| `.tblic` | `TBLC` | license: proves a user owns a realm, checked offline          | v1      | draft  |
| `.tbres` | `TBRS` | resource, undefined                                           | -       | stub   |

standard formats (`.wasm`, `.glb`, `.png`, ...) are stored in packs as-is and aren't
documented here. scripts are plain `.wasm` modules, see [tbman v1](tbman/v1.md#scripts).

## how they fit together

```
root key ─> certificate ─> .tbsig ─> .tbman ─> pack index ─> every chunk of every file
root key ─> certificate ─> .tblic ─> "user X owns realm Y"
```

a `.tbpk` holds one manifest, its signature, its index and the file data. licenses live on
the player's machine, separate from packs. everything in the chain can be checked offline.

## shared rules

read these first. every format builds on them, so no format repeats them.

- [conventions](common/CONVENTIONS.md): byte order, encoding, ids, path rules
- [header](common/HEADER.md): the 24 byte header every terobit file starts with
- [integrity](common/INTEGRITY.md): hashing, keys, signing, revocation, offline rules
- [versioning](common/VERSIONING.md): how formats change without breaking old games

## layout

```
docs/formats/
  README.md            this file
  common/              rules shared by every format
  <ext>/
    REF.md             overview, current version, supported versions, changelog
    v1.md              exact layout of version 1 (frozen once released)
    v2.md              ...
    source.md          optional: the human-written format the tooling compiles from
```

## adding a format

1. extension: `tb` plus 2-3 letters. magic: 4 bytes starting with `TB`
2. create `<ext>/REF.md` and `<ext>/v1.md` from an existing format
3. add a row to the table above and to the magic table in [header](common/HEADER.md#magics)
