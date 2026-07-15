# Arenic

A minimal Bevy 0.19 game. Running it opens the native game window; close the
window to stop the application.

## Requirements

- A current Rust toolchain with Cargo. Install one with
  [rustup](https://rustup.rs/) if needed.
- Graphics drivers that support Bevy's renderer.

## Run

From the repository root:

```sh
cd game
cargo run -p game
```

The first run takes longer because Cargo must compile Bevy and its dependencies.
Running from `game` also keeps Bevy's future asset paths rooted at `game/assets`.

## Validate

Run the repository's ordinary checks from the Cargo workspace:

```sh
cd game
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

## IDE setup

The Cargo workspace manifest is `game/Cargo.toml`. If an IDE does not discover
the project automatically, attach or open that manifest as the Cargo project.
