# terobit_editor

Will be the main editing suite for realms: the client plus tools. Once it exists it
replaces both `tbpk_generator` and the client's `--dev` mode.

## State

Not started. `main` prints "Hello, world!" and the crate has no dependencies. Use
`cargo pkgen` and `cargo rcl` in the meantime.

## Plan

From the [tbpk docs](../../docs/formats/tbpk/REF.md#dev-directory):

- [ ] reuse the client's runtime plugins and add editing tools on top
- [ ] play a dev directory directly, without packing it, the way `--dev` does now
- [ ] a released editor only plays a dev directory for a developer of that `realm_id`,
  checked offline with a signed token, so a shared dev directory can't be used to pirate a
  realm. The token format isn't defined yet
- [ ] build and upload packs with `runtime_assets::pack::writers`
- [ ] when the editor replaces `--dev`, keep the `dev` cargo feature in the code but unused
