# Repository Instructions

## Mandatory Skills

- At the start of every task in this repository, ensure `$bevy-019`, then `$rust-modern`, then `$pragmatic-tiger` are loaded in that precedence order.
- If a mandatory skill's complete contents are already available in the active context and its file has not changed since it was read, treat that skill as loaded. Do not invoke or reread it merely because the user issued another command or began another task in the same context.
- Read only mandatory skills that are missing from the active context, may have changed, or must be restored after a context reset. When more than one must be read, preserve the required `$bevy-019` then `$rust-modern` then `$pragmatic-tiger` order.
- Before every shell command, tool call, code change, or code review, confirm that all three skills are loaded in the active context and apply their relevant guidance to that action. This confirmation does not require rereading an unchanged skill.
- Do not skip a mandatory skill for commands that appear unrelated to Bevy or Rust. If any skill is unavailable, stop and report the problem before running commands or changing files.

## Skill Storage

- Store every custom skill created or maintained for this project under `.agents/skills/<skill-name>` inside this repository.
- Never create, copy, or move a project-specific skill into `$HOME/.codex/skills`, `$HOME/.agents/skills`, or another user-, system-, or machine-level skill directory.
- When using `$skill-creator`, pass the repository's `.agents/skills` directory as the explicit output path.
- Before creating or moving a skill, resolve its destination and verify that it remains inside this repository.
- Use a user- or system-level destination only when the user explicitly requests a skill that applies outside this project.

## Skill Precedence

- Follow the user's request and repository instructions first.
- Let `$bevy-019` exclusively own Bevy APIs, ECS, scheduling, assets, rendering, and framework-specific testing guidance.
- Let `$rust-modern` exclusively own Rust language idioms, types, APIs, arithmetic, errors, code organization, Cargo conventions, and Rust testing mechanics.
- Use `$pragmatic-tiger` only for complementary project-level decisions about risk, bounds, performance budgets, dependencies, invariants, documentation, and technical debt.
- If `$pragmatic-tiger` overlaps or conflicts with either specialized skill, ignore its guidance at that boundary and follow the specialized skill.

## Bevy 0.19 Mandatory Policy

### Version And Verification

- Before writing, editing, generating, or reviewing any `bevy` or `bevy_*` code, confirm that `$bevy-019` is loaded in the active context and has not changed since it was read. Emit only Bevy `0.19.0` APIs.
- Treat copied code, model memory, tutorials, answers, and examples from any other Bevy release as untrusted migration input. Never paste older Bevy code unchanged.
- Treat every forbidden-to-current mapping in `$bevy-019` as one-way: the obsolete form must not be introduced.
- If `$bevy-019` does not cover a symbol or behavior, verify it against the installed `0.19.0` crate source, the [Bevy 0.19 API docs](https://docs.rs/bevy/0.19.0/bevy/), or the official [0.18 to 0.19 migration guide](https://bevy.org/learn/migration-guides/0-18-to-0-19/). Do not guess.
- If the Bevy dependency version changes, stop Bevy implementation work until the version-specific skill and this policy are updated together. Do not silently mix release APIs.

### Non-Negotiable API Rules

1. **Spawn direct components; do not use legacy engine bundle structs.** `Camera2dBundle`, `Camera3dBundle`, `PbrBundle`, `MaterialMeshBundle`, `SpriteBundle`, `NodeBundle`, `TextBundle`, `SpatialBundle`, and similar pre-0.19 convenience bundles are forbidden. Use `spawn((A, B, C))` and let required components supply the rest. Bevy's `Bundle` trait and `#[derive(Bundle)]` still exist in 0.19, so do not claim that all bundles were deleted. A custom bundle is allowed only for a documented, stable component grouping that is reused and cannot be represented more clearly with direct components or required components.
2. **Never attach a bare `Handle<T>` as an entity component.** Use current typed components such as `Mesh3d`, `Mesh2d`, `MeshMaterial3d`, `MeshMaterial2d`, `SceneRoot`, and `AudioPlayer`. Bare typed handles remain valid as resource fields, asset collections, locals, and other non-component data.
3. **Use messages for buffered communication.** Derive `Message`, register with `add_message::<T>()`, write through `MessageWriter::write`, and read through `MessageReader::read`. Do not use legacy `EventReader`, `EventWriter`, `add_event`, `send`, or `iter` patterns for buffered queues. Use direct mutation when a communication boundary adds no value.
4. **Use events only for observers.** Derive `Event` for observer-triggered behavior and `EntityEvent` when an event targets an entity or needs relationship propagation. Observer systems take `On<E>`, never the removed `Trigger<E>` system parameter. Current lifecycle events are `Add`, `Insert`, `Discard`, `Remove`, and `Despawn`; `Replace`, `OnReplace`, `on_replace`, and `#[component(on_replace = ...)]` are forbidden in favor of `Discard`, `on_discard`, and `#[component(on_discard = ...)]`.
5. **Register every system with an explicit schedule.** Use `add_systems(Startup, ...)`, `add_systems(Update, ...)`, `add_systems(FixedUpdate, ...)`, or another typed schedule. `add_system`, `.system()`, and stage APIs are forbidden. Handle input and presentation work in `Update`; run fixed-tick, replay-relevant gameplay simulation in `FixedUpdate`, bridging them with bounded intent or state.
6. **Make ordering explicit only where a dependency exists.** Use system sets, `.before`, `.after`, or `.chain()` for real sequencing requirements. Never rely on plugin registration order, incidental scheduler order, query iteration order, or entity IDs for gameplay outcomes. Do not serialize independent systems unnecessarily.
7. **Author every project color in OKLCH.** Every authored color literal in project code, tests, examples, and documentation must use `Color::oklch(lightness, chroma, hue_degrees)` for an opaque color or `Color::oklcha(lightness, chroma, hue_degrees, alpha)` when alpha is intentional. Other authored color constructors, direct color-space structs, named color constants, palettes, and hex literals are forbidden, including `Color::srgb`, `Color::srgba`, linear color constructors, `Srgba`/`LinearRgba` literals, `Color::WHITE`, and `bevy::color::palettes`. If a Bevy API, asset format, or GPU shader requires another representation, keep the authored source in OKLCH and convert only at that narrow boundary; do not use the boundary representation as the project's authoring format. Perform lighting and blending math in the linear representation required by the renderer, not directly on OKLCH components.
8. **Handle query cardinality and access explicitly.** `Query::single()` and `single_mut()` return `Result`; propagate or handle the error. `get_single` and `get_single_mut` are forbidden. Use the `Single` system parameter only when exactly one match is a real invariant and its validation behavior is desired. Fetch only required data, request `&mut T` only when mutating it, and use `Added<T>` or `Changed<T>` only when their change-detection semantics are intended.
9. **Use current relationships for hierarchy.** Use `ChildOf` and `Children`; read the parent with `child_of.parent()`. Create relationships with `ChildOf`, current hierarchy builders, or `children![...]`. Do not directly mutate `Children`, because it is maintained from the relationship source. `despawn()` recursively despawns `Children` and other linked relationship descendants; legacy `Parent`, `push_children`, and `despawn_recursive` APIs are forbidden.
10. **Treat resources correctly in Bevy 0.19.** `#[derive(Resource)]` also implements `Component`; never derive both `Resource` and `Component` for one type, and never reuse one resource type as ordinary per-entity state. Split those concepts into distinct types. Broad queries such as `Query<Entity>`, `Query<EntityRef>`, or `Query<Option<&T>>` can include abstract resource entities; narrow them with domain components or `Without<IsResource>` when ordinary game entities are intended.
11. **Use current input APIs behind device-neutral intent.** Read keyboard and mouse buttons through `ButtonInput<KeyCode>` and `ButtonInput<MouseButton>`. Read connected controllers by querying `Gamepad` components; legacy `Input<T>` and `Gamepads` resource patterns are forbidden. Hardware adapters may produce shared gameplay intent, but gameplay systems must not encode controller-, keyboard-, or mouse-specific rules. Choose `pressed`, `just_pressed`, and repeat behavior deliberately, and bound the amount of input consumed per simulation step.
12. **Use current rendering, UI, text, scene, and audio components.** Spawn `Camera2d` or `Camera3d`, `Mesh2d` or `Mesh3d`, material wrappers, `Sprite`, `Node`, current text components, `SceneRoot`, and `AudioPlayer` directly. Do not resurrect removed render/UI bundles, old `shape::` mesh paths, or pre-0.19 text internals. Verify specialized rendering APIs against 0.19 rather than extrapolating from core ECS rules.

### ECS And App Design

- Keep `main` limited to application and plugin assembly. Put reusable behavior in focused plugins that own their systems, schedules, messages, events, states, resources, and initialization.
- Store per-entity state in components and unique global state in resources. Use Bevy states for explicit application modes, not unrelated boolean resources.
- Keep rule-critical or replay-critical domain state authoritative outside rendering components. Treat `Transform`, visibility, meshes, materials, text, and other presentation components as synchronized views when deterministic rules depend on the same concept.
- Prefer marker-component presence for binary entity traits. Add a boolean field only when false is meaningfully different from component absence.
- Remember that `Commands` are deferred. Do not assume a spawned, inserted, removed, or despawned entity is visible to later systems without a valid deferred-command boundary and explicit ordering. Prefer direct query mutation when the entity already exists and immediate mutation is required.
- Preserve Bevy parallelism with narrow queries and resources. Resolve query conflicts by modeling access accurately; do not hide conflicts with broad exclusive systems or unnecessary global mutation.
- Use run conditions when the scheduler can skip inactive work. Do not wake a system every frame only to immediately return when a stable condition is false.

### Assets, Features, And Platform Boundaries

- Use stable asset paths and typed handles. Keep required handles in an owning resource or component wrapper, make missing required assets visible, and do not block the main thread waiting for asynchronous loads.
- Treat Bevy feature selection as explicit. In 0.19, `2d` and `3d` do not imply `ui`, and `2d`, `3d`, and `ui` do not imply `audio` when configuring non-default features. Verify every required feature instead of depending on an old feature relationship.
- Do not add target-specific Bevy features, windowing assumptions, asset paths, input behavior, or renderer configuration to shared game logic. Isolate unavoidable platform work behind a narrow plugin or adapter.
- Do not use a successful Windows run to claim support for any other target.

## Repository Scope

- Keep the project target-agnostic unless the user or repository policy explicitly selects supported targets.
- The current local development host is Windows (`x86_64-pc-windows-msvc`). Use it for fast local verification, not as a statement of product support.
- Apply the skills' independent Bevy, Rust, and project-level engineering guidance according to the ownership rules above.
- Keep shared code free of platform assumptions and isolate target-specific integration behind narrow boundaries.
- Do not add or remove target-specific features, dependencies, or configuration without an explicit target requirement.

## Validation

- Run Cargo commands from the repository root unless the task requires another working directory.
- For Rust, Bevy, Cargo, or build-configuration changes, run `cargo fmt --all --check`.
- For Rust or Bevy code changes, run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`.
- Validate ordinary Rust, Bevy, Cargo, or build-configuration changes with `cargo build --workspace` on the current host.
- For documentation-only or repository-policy-only changes, review the scoped diff and validate referenced local paths and external version links. Cargo validation is not required unless the documentation changes executable commands or build configuration.
- Add focused tests for changed gameplay invariants and communication boundaries. Pure rule logic should be testable without a renderer; compile-check Bevy wiring through the owning plugin or app boundary.
- Run a visible or hardware smoke test when rendering, windowing, audio, assets, or input-device behavior materially changes and the environment supports it. State clearly when such a test was not possible.
- Review changed Bevy code for the forbidden legacy forms in this policy before completion, especially when migrating or copying code.
- Use `--target` only when the task or repository policy selects a target, and do not claim compatibility for untested targets.
