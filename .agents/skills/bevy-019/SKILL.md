---
name: bevy-019
description: >-
  Target-agnostic Bevy 0.19 guidance. Use before writing, editing, or reviewing
  Bevy code involving app structure, ECS state, component spawning, schedules,
  messages, events, assets, platform boundaries, or framework testing.
---

# Bevy 0.19

Use Bevy `0.19.0` APIs while preserving target portability. Treat the current
development host as a local verification environment, not a declaration of the
game's supported targets.

## Hard Rules

- Use Bevy `0.19.0` APIs only.
- Keep transient scene state in ECS resources and components. Keep durable game
  progress in a domain or persistence layer rather than render objects.
- Spawn components directly. Do not reintroduce legacy bundle-first patterns.
- Use `Camera2d` and `Camera3d` components directly.
- Use `Mesh2d`, `MeshMaterial2d`, `Sprite`, `Text`, and other current component
  APIs directly.
- Keep shared game and scene logic free of platform assumptions. Isolate
  target-specific integration behind narrow plugins or adapters.
- Do not add target-specific Bevy features or configuration unless the task
  selects that target or the existing project policy requires them.
- Prefer deterministic math and explicit checked or strict arithmetic for
  gameplay rules. Rendering-only interpolation may use ordinary floating-point
  math.

## App Shape

Expose reusable game or scene behavior through focused plugins:

```rust
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene);
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

Keep an executable entry point minimal:

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        .run();
}
```

Let the binary assemble plugins. Keep reusable behavior in the plugins that own
it.

## Components Instead Of Bundles

Prefer direct component tuples:

```rust
commands.spawn((
    Sprite {
        color: Color::srgb(0.2, 0.7, 1.0),
        custom_size: Some(Vec2::new(64.0, 64.0)),
        ..default()
    },
    Transform::from_xyz(0.0, 0.0, 0.0),
));
```

For meshes:

```rust
commands.spawn((
    Mesh2d(meshes.add(Rectangle::new(80.0, 40.0))),
    MeshMaterial2d(materials.add(Color::srgb(0.9, 0.4, 0.1))),
    Transform::from_xyz(0.0, 0.0, 0.0),
));
```

## Scheduling

Use typed schedules and system sets:

```rust
app.add_systems(Startup, setup_scene)
    .add_systems(Update, (advance_animation, sync_scene_state));
```

Use `FixedUpdate` for deterministic gameplay simulation. Use `Update` for
render-facing interpolation and input handling.

## Messages And Events

Use messages for buffered communication across frames:

```rust
#[derive(Message)]
pub struct CardReviewed {
    pub card_id: u64,
}

fn send_review(mut writer: MessageWriter<CardReviewed>) {
    writer.write(CardReviewed { card_id: 7 });
}

fn read_reviews(mut reader: MessageReader<CardReviewed>) {
    for review in reader.read() {
        let _ = review.card_id;
    }
}
```

Register messages on the app:

```rust
app.add_message::<CardReviewed>();
```

Use observed events when an event targets an entity or should trigger
observers:

```rust
#[derive(Event)]
pub struct Selected;

fn on_selected(trigger: On<Selected>) {
    let _entity = trigger.target();
}
```

Choose direct mutation when decoupling does not add value. Use messages,
events, and observers only when the communication boundary is meaningful.

## Assets

Use typed handles and stable asset paths:

```rust
#[derive(Resource)]
pub struct SceneAssets {
    pub card_texture: Handle<Image>,
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SceneAssets {
        card_texture: asset_server.load("cards/card.png"),
    });
}
```

Make missing required assets visible during development and testing. Keep
platform-specific asset loading policy outside shared scene logic.

## Target Discipline

- Treat the build target as an explicit input when a task depends on it.
- Use the current host target for ordinary local checks when no target is
  selected. Do not infer product support from that check.
- Keep shared plugins and systems platform-neutral wherever practical.
- Put target-specific windowing, storage, input, rendering, or lifecycle
  integration behind narrow boundaries.
- Document why a target-specific dependency feature is required before adding
  it.
- Validate each explicitly supported target before claiming compatibility.

## Testing Boundaries

- Test pure gameplay logic independently of rendering and platform integration.
- Compile-check Bevy wiring on each target selected by the task or repository
  policy.
- Use a visible smoke test when rendered output materially matters.
- Treat a successful host run as local evidence only, not cross-target proof.
- Keep renderer tests deterministic. Avoid ambient randomness, real-time
  dependence, and platform-specific asset discovery unless the test owns that
  integration.

## Anti-Patterns

- Inferring the game's supported targets from the developer's operating system.
- Adding platform-specific dependency features without an explicit need.
- Reintroducing component bundle names from older Bevy examples.
- Storing durable progress or settings only in Bevy resources or components.
- Hiding scene setup behind generic abstractions before repeated scene patterns
  justify them.
- Allowing platform integration details to spread through shared systems.
