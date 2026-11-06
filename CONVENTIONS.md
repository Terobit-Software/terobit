# Conventions
This file contains rules and conventions applied throughout this repo when editing.

---

## Workspace Deps
All dependencies should be contained at the workspace level. Crates and binaries should never have different versioned dependencies.

#### /Cargo.toml
```toml
bevy = { version = "0.20.0", default-features = false }
```

#### crate/Cargo.toml
```toml
bevy.workspace = true
```

## Workspace Version
Every crate and binary shares one version, set in `[workspace.package]`. Never give a crate its own version.

#### /Cargo.toml
```toml
[workspace.package]
version = "0.1.0"
```

#### crate/Cargo.toml
```toml
[package]
name = "runtime_example"
version.workspace = true
edition = "2024"
```

## Logging
All logging should be done through tracing. Use `info!`, `error!`, and `warn!`.

## Errors
Errors are typed with [SNAFU](https://docs.rs/snafu). Never return `Result<_, String>` or `Result<_, ()>`.

- Each module that can fail gets its own error enum (`PathError`, `ManifestLoadError`, `PackWriterError`, ...) next to the code that returns it. Don't grow one crate-wide enum.
- Every variant says what was being done and carries the values needed to understand it: paths, ids, sizes, targets. Display messages are lowercase, have no trailing period, and don't repeat the source's message.
- Wrap lower errors with `.context(...)` instead of `map_err(|e| e.to_string())`, so the whole cause chain is kept.
- Use `ensure!` for checks and `.context(...)` on `Option`s instead of `if ... { return Err(...) }` and `ok_or_else`.
- Where an error is finally handled, log the whole chain with `snafu::Report`:

```rust
if let Err(e) = writer.build_tbpk(target) {
    error!("Skipping the {target:?} pack: {}", snafu::Report::from_error(&e));
}
```

```
Skipping the Client pack: failed to load manifest `basic1/manifest.toml`

Caused by these errors (recent errors listed first):
  1: manifest `basic1/manifest.toml` was parsed but failed validation
  2: asset id `self:cube.material` in [assets] is not a valid handle
  3: handle `self:cube.material` has id `cube.material`, but an id must ...
```

- Warnings say what happened, what it means, and what to do about it if anything.
- Add `#[tracing::instrument(skip_all, fields(...))]` to entry points like `build_tbpk` or `read_pack`, so every log inside them carries the target, path, etc.
- `bevy::prelude::*` brings in a `context` method that clashes with SNAFU's. In files that use SNAFU, import only the bevy items you need.

## Unwrap
Minimize the use of unwrap or expect in any code.

#### Do:
```rust
let number = match value {
    Ok(v) => v,
    Err(e) => { 
        tracing::error!("number is not ok, err: {e}");
        return;
    }
};
```

#### Dont:
```rust
let number = value.expect("number is not ok");
```

If code MUST panic when wrong, please use `panic!`. This creates cleaner, and more traceable code. Panics should have custom and verbose languages to your ability. If you cannot control the panic, say for example in a Bevy crate, then you have no control there. Always, when possible, throw an error not a panic. For example:

#### Do:
```rust
match option {
    Some(v) => { /* ... */ },
    None => panic!("value is None")
}
```

#### Dont:
```rust
option.unwrap()
```

## Pull requests
These are enforced by the workflows in `.github/workflows`.

- **Bump the version.** Every PR into `main` must change the `[workspace.package]` `version` in `/Cargo.toml`. The `Version check` workflow fails if it still matches `main`.
- **Pass CI.** `cargo fmt --all --check`, `cargo check --workspace --all-targets --profile ci`, `cargo nextest run --workspace --cargo-profile ci`, and `cargo nextest run -p client_dev --features dev --cargo-profile ci`. Run `cargo fmt --all` before pushing.
- **Workflow changes need review** from `@Terobit-Software/head-development` (see `.github/CODEOWNERS`).
- **Nightlies are automatic.** Every night at midnight Central, `release/nightly/MM.DD.YYYY` is cut from `main` and older nightly branches are deleted. Don't push to them.
