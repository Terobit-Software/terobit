# terobit file formats

reference for every file terobit reads or writes. the client, server and editor all
follow these docs, so the docs are the source of truth. if code and docs disagree, one of
them is a bug.

## formats

| ext      | name      | what it is                                                     | current | status |
|----------|-----------|----------------------------------------------------------------|---------|--------|
| `.tbpk`  | pack      | the container a game ships in, one for client, one for server  | -       | draft  |
| `.tbman` | manifest  | what is in a pack, who owns it, hashes of every file           | v1      | draft  |
| `.tbsig` | signature | signature over a manifest                                      | v1      | draft  |
| `.tbres` | resource  | undefined                                                      | -       | stub   |

scripts are plain `.wasm` files listed in the manifest. there is no terobit wrapper around
them because wasm already has its own header and version (see
[tbman v1](tbman/v1.md#scripts)). client and server scripts ship in separate packs (see [tbpk](tbpk/REF.md#client-and-server-packs)).

## shared rules

read these first. every format builds on them, so no format repeats them.

- [conventions](common/CONVENTIONS.md): byte order, types, strings, ids, paths
- [header](common/HEADER.md): the 24 byte header every binary terobit file starts with
- [integrity](common/INTEGRITY.md): hashing and signing
- [versioning](common/VERSIONING.md): how formats change without breaking old packs

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

1. pick a 3-4 letter extension starting with `tb` and a 4 byte magic (see [header](common/HEADER.md#magics))
2. create `<ext>/REF.md` and `<ext>/v1.md` from an existing format
3. add a row to the table above and the magic table in [header](common/HEADER.md#magics)
