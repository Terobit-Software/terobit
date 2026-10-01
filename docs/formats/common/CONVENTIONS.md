# conventions

applies to every format unless that format's doc says otherwise.

## byte order

little endian, always. every platform terobit targets is little endian, so reading is a
plain copy and needs no byte swapping.

## body encoding

file bodies are encoded with [postcard](https://docs.rs/postcard) from a versioned rust
struct. postcard is compact, fast, and needs no hand-written parser. each version doc
gives the struct, and the struct **is** the layout: field order is wire order.

postcard rules worth knowing when reading a version doc:

| rust type          | on the wire                                         |
|--------------------|-----------------------------------------------------|
| `u8`, `i8`         | 1 byte                                              |
| `u16`..`u64`       | varint (small numbers take fewer bytes)             |
| `[u8; N]`          | N raw bytes, no length                              |
| `String`, `Vec<T>` | varint length, then the items                       |
| `Option<T>`        | `0` for none, `1` then `T` for some                 |
| `enum`             | varint variant index, then the variant's fields     |

postcard is not self-describing. adding, removing or reordering a field changes the layout,
so it needs a new version (see [versioning](VERSIONING.md)).

the fixed [header](HEADER.md) is the only part not encoded with postcard, so a reader can
always identify a file and its version without knowing the body layout.

## strings

utf-8, never null terminated. length comes from the encoding.

## ids

all platform ids (`realm_id`, `owner_id`, user ids) are `u64`. `u32` runs out at ~4 billion,
and a ugc platform can pass that for things like assets or sessions. varint encoding means
small ids still cost 1-2 bytes, so there is no size reason to use smaller types.

ids are never 0. use `Option` for "no id".

## paths inside a pack

- utf-8, `/` separated, relative to the pack root (`models/castle.glb`)
- no leading `/`, no `..` segments, no `.` segments, no empty segments, no `\`
- no symlinks, devices or other special files
- compared byte for byte (case sensitive), but two paths that differ only in case are
  rejected so packs behave the same on case-insensitive filesystems

readers **must** reject a pack that breaks any of these. this blocks path traversal attacks
where a pack writes outside its own directory.

## asset ids

`namespace:name`, both parts lowercase `a-z`, `0-9`, `_`, `-`, `/`. the namespace is the
origin of the asset (for example `self` for assets in this pack). names are what scripts use
to refer to assets, so scripts never touch file paths directly.
