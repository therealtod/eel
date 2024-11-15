# Eel

A Hanabi bot written in Rust, built around tree search and H-Group conventions.


## Building

```bash
cargo build                         # debug
cargo build --release               # optimised
```

## Testing

```bash
cargo test                 # all tests
cargo test <name>          # filter by name substring
cargo test --test '*'      # integration tests only
cargo clippy               # lint
cargo fmt                  # format
```
