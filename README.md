# vimux

A Linux keyboard-control tool written in Rust.

## Development

Enter the Nix development shell directly, or allow direnv to load it automatically:

```bash
nix develop
# or, once:
direnv allow
```

Build and inspect the accessibility tree with Cargo or Nix:

```bash
cargo run -- inspect
nix run -- inspect
```
