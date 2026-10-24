# terobit_server

Will run a realm's server pack and handle client connections for:

- official servers, one docker image per realm
- developers testing their realm
- community hosts
- players hosting for p2p or offline play, when the realm allows it

## State

Not started. `main` prints "Hello, world!" and the crate has no dependencies.

## Plan

Most of this comes straight from the [tbpk docs](../../docs/formats/tbpk/REF.md).

- [ ] build on `runtime_core` and `runtime_logging` with no window or render plugins
- [ ] a `server_dev` crate in `crates/server`, like `client_dev`: `--dev` reads only shared
  and `[server.*]` entries from a dev directory, never logs in, and only accepts a dev
  client on the same machine
- [ ] load `.server.tbpk` archives and reject any whose manifest `target` isn't `Server`
  (needs F1)
- [ ] networking. Reject clients whose `pack_version` doesn't match the server's
- [ ] run server scripts in `runtime_sandbox` (F6)
- [ ] check ownership of `Licensed` realms through the platform. Servers don't use
  `.tblic`, that's only for offline clients
- [ ] a docker image per realm for official hosting
