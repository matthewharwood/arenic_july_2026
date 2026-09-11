# Arenic

A minimal Bevy 0.19 game.

[Play Arenic in your browser](https://matthewharwood.github.io/arenic_july_2026/)
or run it locally in a native window. Connect a controller and press a button
to activate it, use the D-pad to move, and click the left stick to change cameras.
The browser version requires WebGL 2 and hardware acceleration.

## Arena HUD

The native and browser versions share the same HUD and OKLCH design tokens.

| Control | Action |
| --- | --- |
| Tab / Shift+Tab | Select the next / previous living character |
| [ / ] | Visit the previous / next arena |
| 1 / 2 / 3 / 4 | Four fixed ability slots, mapped to the selected class |
| H | Open or close the controls guide |
| R | Reserved recording control |
| Controller D-pad | Move one tile |
| Controller left-stick click | Switch arena / shoulder camera |

Click roster icons and arena cells to navigate directly. Forty roster positions
remain visible; extra characters appear in the reserve tray, and selecting one
keeps it visible in the main roster. Empty positions use X, fallen characters use
a skull, and the current selection is blue. The 3×3 arena map uses black for the
current arena, X for empty rosters, and a red dot for an alert. Raid difficulty is
read-only Normal.

Names are stable ID-derived pairs from 100 first names and 100 surnames, with
10,000 unique combinations before display names repeat. Buffs and debuffs count
down on fixed ticks; tags show debuffs first, then buffs, with the soonest expiry
first within each group. HP stays blue, XP green, gains green, and damage and
harmful effect durations red. Stat bars ease toward changed values, while brief
XP/HP numbers float and fade beside the bars.

This iteration uses sample rosters and one world hero preview. Ability execution,
recording, level-up rules, persistence, and global chat are not implemented. In
the controls guide, **Preview +9 XP / -9 HP** demonstrates stat feedback explicitly
without attaching simulated combat effects to ability buttons. Reloading resets
the sample data. The stat-message boundary accepts at most 64 resolved changes
per fixed tick; future combat producers must respect that contract. Presentation
is separately bounded to eight concurrent floating labels.

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
