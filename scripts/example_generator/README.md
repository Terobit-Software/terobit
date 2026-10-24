# example_generator (`cargo regen`)

Writes example dev directories full of generated files, for testing and benchmarking
`cargo pkgen` and the chunker on packs bigger than the hand-made examples.

## Run

```sh
cargo regen                                  # every type at its default count
cargo regen --type mix --count 500 --auto    # one example, then pack it
cargo regen --type random --min 1 --max 65536 --entropy med
```

| flag        | value                                                      | default            |
|-------------|------------------------------------------------------------|--------------------|
| `--type`    | `xsmall`, `small`, `large`, `xlarge`, `mix`, `xmix`, `random` | all of them     |
| `--count`   | files per side, shared files count for both. Needs `--type` | per type, below   |
| `--min`     | smallest file in bytes, `--type random` only               | 1                  |
| `--max`     | largest file in bytes, `--type random` only                | 1 MiB              |
| `--entropy` | `low`, `med`, `high`: how well the files compress          | `high`             |
| `--auto`    | run `cargo pkgen` on each example afterwards               | off                |

| type     | file sizes                 | default count |
|----------|----------------------------|---------------|
| `xsmall` | 1 - 64 B                   | 500           |
| `small`  | 65 B - 4 KiB               | 250           |
| `large`  | 256 KiB + 1 B - 4 MiB      | 25            |
| `xlarge` | 8 - 16 MiB                 | 25            |
| `mix`    | `small` and `large`        | 100           |
| `xmix`   | every class above          | 100           |
| `random` | `--min` to `--max`         | 50            |

Examples go to `tests/examples/source_to_tbpk/generated/<name>/`, which is gitignored.
The name comes from the settings (like `mix_100` or `small_250_low`), so the same settings
always map to the same folder. An example that already exists is skipped, so delete its
folder to regenerate it. With `--auto`, packs go to `out/tbpk_generator/<name>/`.

## How it works

- about half the files are shared (`[assets]`), the rest are split between
  `[client.assets]` and `[server.assets]`, 20 files per folder
- file bytes come from a blake3 stream keyed by the example's settings and the file's
  path, so the same settings always produce the same files
- each example is written to `<name>.tmp` and renamed when it's done, so a half-written
  example is never mistaken for a finished one

## TODO

- [ ] rewrite it, the file's first line says so (C7)
- [ ] add a size class between 4 KiB and 256 KiB. Those are the files that share small-file
  chunks, and only `--type random` makes them now (C7)
- [ ] switch to clap so `--help` works (C6)
