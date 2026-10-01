# common header

every binary terobit file starts with this 24 byte header. a reader can identify any
terobit file, and decide whether it can read it, from these bytes alone.

| offset | size | type      | field          | notes                                                    |
|--------|------|-----------|----------------|----------------------------------------------------------|
| 0      | 4    | `[u8; 4]` | `magic`        | identifies the format, see [magics](#magics)             |
| 4      | 2    | `u16`     | `version`      | format version, starts at 1                              |
| 6      | 2    | `u16`     | `flags`        | reserved, must be 0 in v1 of every format                |
| 8      | 4    | `u32`     | `header_len`   | bytes from file start to body start, 24 for now          |
| 12     | 4    | `u32`     | `reserved`     | must be 0                                                |
| 16     | 8    | `u64`     | `body_len`     | body size in bytes                                       |
| 24     | -    | -         | body           | postcard encoded, layout depends on `magic` + `version`  |

the header is fixed width little endian, **not** postcard, so it never changes shape.

## reading

1. read 24 bytes. fewer than 24 means the file is invalid
2. `magic` doesn't match the expected format: reject
3. `version` newer than this reader supports: reject with "update terobit to open this"
4. `version` older than the oldest supported: reject with "this pack is too old"
5. `flags` has bits this reader doesn't know: reject
6. skip to `header_len` (lets a future header grow without breaking old readers)
7. read exactly `body_len` bytes. trailing bytes after the body are invalid
8. decode the body with the decoder for `version`, then migrate to the latest in-memory struct

## magics

| magic  | format                       |
|--------|------------------------------|
| `TBMN` | [manifest](../tbman/REF.md)  |
| `TBSG` | [signature](../tbsig/REF.md) |
| `TBRS` | [resource](../tbres/REF.md) (reserved) |

`.tbpk` has no magic because it is a standard archive. see [tbpk](../tbpk/REF.md).
