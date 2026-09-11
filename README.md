# Arenic

A minimal Bevy 0.19 game.

[Play Arenic in your browser](https://matthewharwood.github.io/arenic_july_2026/)
or run it locally in a native window. Connect a controller and press a button
to activate it, use the D-pad to move, and click the left stick to change cameras.
The browser version requires WebGL 2 and hardware acceleration.

## Requirements

- Rust with Cargo. Install [rustup](https://rustup.rs/) if needed; the repository's
  `rust-toolchain.toml` selects stable Rust 1.97.1 and its check tools.
- Graphics drivers that support Bevy's renderer.

## Run

From anywhere inside the repository:

```sh
cargo run -p game
```

The first run takes longer because Cargo must compile Bevy and its dependencies.
Cargo discovers the workspace manifest by searching parent directories up to the
repository root.

## GitHub Pages

The [GitHub Actions workflow](https://github.com/matthewharwood/arenic_july_2026/actions)
checks formatting, Clippy, tests, and the native build, and builds the game for
`wasm32-unknown-unknown`. A push to `main` publishes to GitHub Pages only after
both validation jobs pass. Pull requests run the same checks without deploying.

The browser shell is `web/index.html`. The build packages it with the generated
JavaScript and WebAssembly, using the `wasm-bindgen` version in `Cargo.lock`.
`revision.txt` on the live site identifies the deployed commit; the workflow
checks it after deployment. GitHub Pages must use **GitHub Actions** as its
build source in the repository settings.

## Validate

Run the repository's ordinary checks from anywhere inside the repository:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

## IDE setup

The Cargo workspace manifest is `Cargo.toml` at the repository root. If an IDE
does not discover the project automatically, attach or open that manifest as
the Cargo project.
