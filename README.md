# Terobit
Terobit is a game creation engine and platform focused on multiplayer games. It is designed with security, speed, and customization in mind. 

## Libraries and Binaries
### Runtime
The main library, required by the client and server (coming soon). It handles nearly everything, the client just adds client features.

### Client
The main client binary, uses the runtime library. Extends the runtime with client-specific features.

## Requirements
- Python 3+ ([installation instructions](https://www.python.org/downloads/))
- Diesel CLI ([installation instructions](https://diesel.rs/guides/getting-started))
- Rust 1.98+ reccommended ([installation instructions](https://www.rust-lang.org/tools/install))

## Building and Running 
Running:
```
python scripts/run_client.py
```
Test:
```
cargo test -p runtime
```


## Platform Support
| Platform | Status  |
|----------|---------|
| Windows  | yes     |
| macOS    | yes     |
| Linux    | unknown |
