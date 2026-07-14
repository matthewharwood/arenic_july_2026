---
name: rust-modern
description: >-
  Modern, target-portable Rust 2024 guidance. Use when writing, editing,
  reviewing, or scaffolding Rust code involving deterministic game logic,
  Cargo workspaces, clippy-clean APIs, error handling, portability, tests, or
  current stable Rust idioms.
---

# Rust Modern

Write clear Rust 2024 code that remains portable unless a task explicitly
selects a target. When Bevy APIs are involved, read `bevy-019` first; that skill
owns framework mechanics. This skill owns Rust language and code-judgment rules
around those APIs.

## Baseline

- Use `edition = "2024"` for new crates.
- Set `resolver = "3"` explicitly in a virtual workspace root.
- Prefer stable Rust unless the repository explicitly opts into nightly for a
  specific crate. Do not introduce `#![feature(...)]` casually.
- Keep domain and gameplay crates target-portable when practical. Put platform
  APIs behind narrow adapters.
- Prefer repository-provided deterministic commands for repeated workspace
  checks. Do not invent a build tool or task runner that the repository does
  not use.
- Keep formatting, clippy, documentation, and test commands compatible with the
  targets selected by repository policy or the current task.

## Rust 2024 Rules

### Return-position `impl Trait`

Edition 2024 captures all in-scope generics and lifetimes by default.

Use `+ use<>` when a helper returns `impl Iterator`, `impl Fn`, or another
opaque type that does not borrow its inputs:

```rust
fn ids(&self) -> impl Iterator<Item = u32> + use<> {
    0..4
}
```

Add only the parameters that are truly captured, such as `use<'a, T>`.

### Temporary scopes

Do not assume a guard or borrow created in an `if let` scrutinee or a block tail
expression lives to the end of the block. Bind it first if it must stay alive.

### Let chains

Flatten nested optional lookups:

```rust
if let Some(player) = player
    && let Some(target) = player.target
    && target.alive
{
    attack(player, target);
}
```

This is usually clearer and keeps clippy's collapsible-if checks quiet.

### Match guards

Use `if let` guards for fallible secondary lookups in a match arm:

```rust
match action {
    Action::Use(id) if let Some(item) = inventory.get(id) => use_item(item),
    _ => {}
}
```

## Current Stable Idioms

- Use `get_disjoint_mut` for two or more mutable references into one slice,
  `Vec`, or `HashMap` by index or key. Do not use repeated indexing or unsafe
  split workarounds.
- Use `Vec::extract_if` when removing and collecting matching elements.
- Use `as_chunks::<N>()` for fixed-size slice groups such as RGBA pixels or quad
  vertices.
- Use `[value; _]` inside function bodies when the return type or const already
  fixes the array length.
- Use `LazyLock` for compute-once global tables. Avoid hand-rolled `OnceLock`
  plumbing unless initialization needs custom error behavior.
- Use `core::hint::cold_path()` at the top of rare branches in hot loops.
- Use `core::range::Range` for ranges stored in `Copy` components or resources.
- Use `assert_matches!` in tests instead of `assert!(matches!(...))`.

## Deterministic Game Logic

- Run recorded or replayable simulation on fixed ticks, not variable frame
  deltas.
- Do not let deterministic state depend on wall-clock time, unseeded RNG, or
  `HashMap` iteration order.
- Pick an explicit overflow policy in deterministic math:
  - Use `strict_*` when overflow is a bug and should fail loudly.
  - Use `wrapping_*` when wraparound is intentional and replay-stable.
- Avoid bare `+`, `-`, and `*` for tick counters, grid indices, resource totals,
  replay streams, and deterministic accumulators.
- Store intent and inputs, then derive results. A replay should be a pure
  function of recorded input and tick.
- Keep transition handlers idempotent: running them twice should either be
  impossible by construction or produce the same final state.

## Error Handling

- Propagate errors with `Result` and `?`; do not use `.unwrap()` in application
  or system code.
- Use `.expect("invariant: ...")` only for a real invariant and explain it in
  the message.
- Use `thiserror` for domain and library error enums.
- Use `anyhow` or miette-style reports only at binary, build-tool, or CLI edges
  where the caller wants context rather than a stable error type.
- Do not silently drop a `Result`. If fire-and-forget behavior is intentional,
  make that intent explicit and handle or log the failure where it matters.

## API Shape

- Prefer plain data types with public fields when there is no invariant to
  protect.
- Keep fields private only when the type enforces a real invariant.
- Use finite enums for compile-time-known sets. Prefer exhaustive `match` arms
  over catch-all `_` when future variants should force a code review.
- Consume `self` for one-way transforms and builders that should not leave the
  source usable.
- Return views, iterators, slices, or references rather than allocating a
  `Vec`. If a function allocates, make that behavior clear from its name or
  documentation.
- Do not suffix types with `Component`, `Data`, or `Tag` unless two distinct
  concepts need disambiguation.
- Import common types at module scope. Avoid long inline paths in function
  bodies unless they clarify a rare conflict.

## Bevy-Adjacent Rust Judgment

- Use `bevy-019` for all Bevy APIs before touching Bevy code.
- Mutate stored entity state directly through narrow queries when decoupling is
  not buying anything.
- Use messages, events, and observers only when the communication boundary
  earns its keep.
- Shape query signatures as scheduling contracts: fetch only what is needed and
  ask for `&mut T` only when the system mutates `T`.
- Gate systems with run conditions instead of guard-and-return when the
  scheduler can skip the work.
- Model global state as resources and per-entity state as components. Avoid
  global mutable state.
- Prefer small cohesive systems by behavior, not tiny fragments that all mutate
  the same component and force ordering.
- Keep async work at asset loading, task-pool, or platform-adapter boundaries,
  not inside ordinary ECS systems.

## Target Portability

- Do not infer a product target from the current development host.
- Keep portable crates free of operating-system and runtime-specific APIs.
- Isolate unavoidable platform integration with narrow modules, traits, or
  `#[cfg(...)]` adapters.
- Put target-specific dependencies in target-specific Cargo sections when
  practical.
- Use portable path and I/O abstractions in shared code.
- Treat serialized, persisted, and inter-process data as untrusted at the
  boundary regardless of platform.
- Run target-specific checks only for targets selected by the task or repository
  policy. Use the host for ordinary local feedback when no target is selected.

## Tests

- Unit-test platform-independent domain logic on the host with `cargo test`.
- Add target-specific tests only for behavior that genuinely depends on that
  target.
- Prefer property tests for math, scheduling, parser, and deterministic replay
  invariants when the state space is broad.
- Assert behavior and boundary contracts rather than private helper details.
- Keep public rustdoc examples compiling.
- Do not treat a successful host test as evidence for untested targets.

## Avoid

- Do not use `generic_const_exprs` in shipping code.
- Do not use `specialization`.
- Do not add `static mut`.
- Do not rely on `HashMap` iteration order for gameplay outcomes.
- Do not introduce nightly features unless repository policy and the crate
  explicitly require them.
- Do not hand-roll iterator state machines before considering current stable
  iterator tools. Do not use nightly `gen` blocks unless explicitly enabled.
