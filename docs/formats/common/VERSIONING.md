# versioning

each format has its own version number in its [header](HEADER.md). formats are versioned
independently: a new manifest version doesn't bump the signature version.

## draft vs released

a version is **draft** until the first public release that can read it. a draft version can
be edited freely, no bump needed. everything is draft right now.

once released, a version doc is **frozen**: `vN.md` never changes again except for typos and
clarifications. any layout change goes into `vN+1.md`.

## what needs a new version

anything that changes bytes on the wire:

- adding, removing, renaming the type of, or reordering a field
- adding an enum variant (old readers can't decode the new index)
- changing what a field means

because postcard isn't self-describing, there is no "optional new field" trick. bump it.

## reading old versions

readers keep a decoder for every supported version. each old decoder converts to the next
version, so only one migration is written per bump:

```
v1 bytes -> ManifestV1 -> ManifestV2 -> ManifestV3 (latest, used everywhere)
```

only the loader knows old versions exist. the rest of the engine sees the latest struct.

## writing

writers always write the latest version. there is no writing old versions.

## golden files

every released version of every format has sample files committed to the repo, along with
the values they should decode to. a test decodes each one and checks the values, and stays
until that version is dropped. if one fails, either a decoder broke or a layout changed
without a version bump, and both are bugs.

while a version is still draft, its golden files can be regenerated freely.

## things that never change

some values are baked into data that outlives any version and can never be edited, only
replaced with new ones:

- hash context strings (see [integrity](INTEGRITY.md#hashing))
- magics (see [header](HEADER.md#magics))
- the zstd version and level of a released manifest version

## dropping support

each `REF.md` lists the oldest version still read. raising it needs every published pack on
that version to have been rebuilt, which the platform can do server side since it has the
sources. record the change in that format's changelog.

## REF.md template

```markdown
# <name> - .<ext>

<one paragraph: what it is and who reads it>

current version: v1 (draft)
oldest supported: v1

## versions

- [v1](v1.md)

## changelog

- v1: initial version
```
