# Terobit Client

The main game client. It loads, runs, handles, client packs.
Depends on the runtime crates. Also depends on client-specific
crates.

## How to Run

You can run the client by entering `cargo xtask run client` or
`cargo x r cl`, which do the same thing. It is not recomended to
run directly.

If you are on Linux, please ensure you have the necessary system
libraries:
`libudev-dev libasound2-dev libxkbcommon-dev libwayland-dev`

## Flags

| flag | desc                                                    |
|------|---------------------------------------------------------|
| `--dev` | runs the client in developer mode, allows use of dev
directory instead of a `.tbpk`. |
| `--pack-file <path>` | the path to the `.client.tbpk` file or dev directory if `--dev` is enabled. |

Please note running client with `cargo xtask run client --final` strips the `--dev` flag and `dev` cargo feature flag so it will refuse the `--pack-file` flag.
