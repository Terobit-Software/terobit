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

postcard is not self-describing. adding, removing or reordering a field or enum variant
changes the layout, so it needs a new version (see [versioning](VERSIONING.md)).

exceptions, all fixed width little endian so they never change shape:

- the [common header](HEADER.md)
- the [tbpk table of contents](../tbpk/v1.md#table-of-contents), so a reader can seek
  without decoding anything
- the byte strings that get signed (see [integrity](INTEGRITY.md)), so there is exactly one
  way to rebuild them

## strings

utf-8, never null terminated. length comes from the encoding.

## ids

all platform ids (`realm_id`, `owner_id`, `user_id`) are `u64`. varint encoding means small
ids still cost 1-2 bytes, so there is no size reason to use smaller types. ids are never 0.

## paths inside a pack

a path is 1-16 segments joined by `/`, at most 255 bytes in total.

each segment:

- is 1-64 bytes, using only `a-z`, `0-9`, `.`, `_`, `-`
- doesn't start or end with `.` (rules out `.`, `..`, hidden files, and the trailing dots
  windows strips)
- isn't a windows reserved name, with or without an extension: `con`, `prn`, `aux`, `nul`,
  `com1`-`com9`, `lpt1`-`lpt9` (so `con.png` is rejected too)

so: no uppercase, no spaces, no unicode, no `\`, no leading `/`, no empty segments.

this makes every path mean the same file on windows, macos and linux. there are no case
or unicode normalization questions, and no path can point outside the pack.

readers must reject a pack with any invalid path. the pack tool and the dev loader enforce
the same rule, so mistakes show up during development and not at publish.

the rule starts strict on purpose: allowing more characters later doesn't break existing
packs, but tightening it would.

## asset and script ids

`namespace:name`. both parts use `a-z`, `0-9`, `_`, `-`, and the name may also use `/`.

`self` is the only namespace in v1 and means "this pack". other namespaces are reserved
for later, such as depending on assets from another realm. scripts refer to assets by id,
never by file path.
