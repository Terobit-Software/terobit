# terobit_client

The game client. It will load a realm's client pack and run it.

## Run

The usual way, which builds, copies to `out/client` and runs with the dev flags:

```sh
cargo rcl
```

Or directly:

```sh
cargo run -p terobit_client --profile release-dev -- --dev --pack-file tests/examples/source_to_tbpk/basic1
```

On Linux you need the same system libraries as CI:
`libudev-dev libasound2-dev libxkbcommon-dev libwayland-dev`.

## Flags

| flag                 | what it does                                                                 |
|----------------------|------------------------------------------------------------------------------|
| `--dev`              | developer mode: lets the client read a dev directory in place of a `.tbpk` |
| `--pack-file <path>` | the `.tbpk` file or dev directory to load. Only read with `--dev` for now. Extra paths after the first are ignored with a warning |

`--dev` should only exist in builds with a `dev` cargo feature. That feature doesn't exist
yet, so every build accepts it (B8).

## How it works

`main` starts logging, then builds a Bevy app from the runtime plugin groups in order:
core, input, scene, window, render, ui, animation, audio, picking. Client plugins
(`ClientDevPlugin`) go last because they read resources the runtime adds. See
[plugin order](../../docs/ARCHITECTURE.md#plugin-order).

## State

- opens a window with rendering, UI, audio, input and picking. Nothing spawns a camera or
  scene yet, so it's empty
- with `--dev`, the dev directory's shared and client assets can be loaded with `PackAssets`,
  but nothing in the client loads any yet
- a `.tbpk` file is only checked to be at least 24 bytes (F1)
- without `--dev`, no pack is loaded at all (F4)
- no login, licenses, networking or scripts

## TODO

- [ ] read `.tbpk` archives (F1) and load them without `--dev` (F4)
- [ ] put `--dev` behind a `dev` cargo feature (B8)
- [ ] check licenses for `Licensed` realms (F3)
- [ ] run client scripts (F6)
- [ ] `cargo rcl` copies `bins/terobit_client/assets`, which doesn't exist. Add it once the
  client has bundled assets (fonts, UI), or drop that step
