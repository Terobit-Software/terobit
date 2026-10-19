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

## Logging
All logging should be done through tracing. Use `info!`, `error!`, and `warn!`.

## Unrwap
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
let value.expect("number is not ok");
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
