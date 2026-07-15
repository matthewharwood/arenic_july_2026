# Bevy 0.19 Recommended Syntax, Features, and Optimization Guide

> Review status: researched against every official Bevy release announcement from 0.1 through 0.19, the official migration guides, the Bevy 0.19.0 API documentation, and the installed 0.19.0 crate source on 2026-07-14.

This is a **current Bevy 0.19.0 guide**, not a catalog of syntax to copy from old versions. The release history explains where today's features came from; every recommendation table uses APIs and practices that are applicable to Bevy 0.19.0.

## Recommendation key

| Label | Meaning |
|---|---|
| **Default** | Start here unless the design has a concrete reason not to. |
| **Use when** | Recommended for the stated problem, not universally. |
| **Measure first** | Valid current API, but its benefit depends on the actual workload. |
| **Avoid** | Obsolete, misleading, or normally inferior in Bevy 0.19. |

## Current baseline

| Concern | Bevy 0.19 recommendation | When | How |
|---|---|---|---|
| Version | Pin the project to the Bevy 0.19 line and verify exact APIs against 0.19.0. | Always. | `bevy = "0.19"`; keep `Cargo.lock` for applications. Do not copy examples from another Bevy release unchanged. |
| Rust edition | Use Rust 2024. | New or migrated 0.19 projects. | Set `edition = "2024"` in each package. |
| App entry point | Keep `main` as app/plugin assembly. | Always. | Add `DefaultPlugins`, focused game plugins, then `run()`. Let each plugin own its systems, resources, messages, states, and initialization. |
| Entity construction | Spawn current components directly. | Small, local, or dynamic entity construction. | `commands.spawn((Camera3d::default(), Transform::default()))`. Required components supply engine invariants. |
| Declarative composition | Use `bsn!`, `bsn_list!`, and scene functions. | Reusable entities, hierarchies, UI, asset-backed presentation, or compositions that are easier to review declaratively. | Return `impl Scene`/`impl SceneList`, then call `.spawn()` as a system or use `CommandsSceneExt::spawn_scene`. |
| Schedules | Register every system with a typed schedule. | Always. | `add_systems(Startup, setup)`, `add_systems(Update, input)`, or `add_systems(FixedUpdate, simulate)`. |
| Buffered communication | Use messages. | Multiple producers/consumers, buffered intent, or a schedule boundary. | `#[derive(Message)]`, `add_message`, `MessageWriter::write`, `MessageReader::read`. |
| Reactive communication | Use events with observers. | Immediate reactions, entity-targeted behavior, propagation, or lifecycle observation. | `#[derive(Event)]`/`#[derive(EntityEvent)]`, trigger it, and observe with `On<E>`. |
| Hierarchy | Use relationships. | Parent/child transforms, ownership, UI trees, or other graph relationships. | `ChildOf`, `Children`, `children![...]`; read the parent with `child_of.parent()`. |
| Render assets | Attach typed render components, not bare handles. | Rendered entities. | `Mesh3d`, `Mesh2d`, `MeshMaterial3d`, `MeshMaterial2d`, `WorldAssetRoot`, `AudioPlayer`. |
| Performance | Measure before specializing. | Before every optimization pass. | Add diagnostics, capture representative workloads, change one bounded factor, then remeasure CPU, GPU, frame time, memory, and loading behavior. |

## What all Bevy releases contributed

The historical column is context only. The last column states the **current 0.19 recommendation**.

| Release | Durable milestone | Current 0.19 use |
|---|---|---|
| [0.1](#bevy-01) · [notes](https://bevy.org/news/introducing-bevy/) | Data-driven ECS, plugins, 2D/3D rendering, UI, assets, scenes, and render graphs established Bevy's core direction. | Keep gameplay data in components/resources and organize behavior into focused plugins. Use the current render schedules, not the original render-graph API. |
| [0.2](#bevy-02) · [notes](https://bevy.org/news/bevy-0-2/) | Task pools, early web support, parallel queries, gamepads, generational entity IDs, and unified transforms. | Treat `Entity` as an opaque generational ID, use current `Transform`/`GlobalTransform`, and parallelize only measured, sufficiently large workloads. |
| [0.3](#bevy-03) · [notes](https://bevy.org/news/bevy-0-3/) | Mobile/touch, asynchronous assets, glTF scenes, and substantial ECS/query ergonomics. | Keep asset loading asynchronous, isolate platform input, and use typed current scene/render components. |
| [0.4](#bevy-04) · [notes](https://bevy.org/news/bevy-0-4/) | WebGL2, live shader reload, and stronger cross-platform application behavior. | Keep shared logic target-neutral; validate renderer and asset behavior on every selected target. |
| [0.5](#bevy-05) · [notes](https://bevy.org/news/bevy-0-5/) | Physically based rendering and `StandardMaterial` became central, with improved glTF support. | Use `StandardMaterial` for ordinary PBR; choose unlit/custom materials only when the visual model calls for them. |
| [0.6](#bevy-06) · [notes](https://bevy.org/news/bevy-0-6/) | The modern renderer architecture, render world, WGSL materials, and frustum culling arrived. | Use 0.19 render schedules/systems for custom passes; preserve automatic culling and typed materials. The old `.system()` conversion is obsolete. |
| [0.7](#bevy-07) · [notes](https://bevy.org/news/bevy-0-7/) | Animation, compressed textures, compute shaders, render-to-texture, `ParamSet`, multi-entity queries, and reactive power modes expanded. | Use these only for their concrete problem; use `ParamSet` to express otherwise-conflicting accesses without broad exclusive systems. |
| [0.8](#bevy-08) · [notes](https://bevy.org/news/bevy-0-8/) | `Material`/`AsBindGroup`, camera-driven rendering, shader imports, inherited visibility, and Taffy UI matured. | Derive current material bindings, use cameras to define views, and rely on current `Visibility`/`InheritedVisibility` and `Node` UI. |
| [0.9](#bevy-09) · [notes](https://bevy.org/news/bevy-0-9/) | HDR, tonemapping, bloom, FXAA, a new scene format, code-built scenes, and improved spawning. | Enable post-processing per camera when the art direction and GPU budget justify it. Prefer current BSN or direct component construction over old scene/spawn syntax. |
| [0.10](#bevy-010) · [notes](https://bevy.org/news/bevy-0-10/) | Schedule v3, cascaded shadows, environment-map lighting, prepasses, pipelined rendering, and windows as entities. | Express real ordering with sets/constraints, query windows as entities, and enable expensive lighting/prepasses only where they produce visible value. |
| [0.11](#bevy-011) · [notes](https://bevy.org/news/bevy-0-11/) | Schedule-first `add_systems`, `FixedUpdate`, gizmos, ECS audio, UI grid/borders, and WebGPU support. | Use typed schedules, put replay-critical simulation in `FixedUpdate`, use gizmos for debug visualization, and spawn `AudioPlayer` for playback. |
| [0.12](#bevy-012) · [notes](https://bevy.org/news/bevy-0-12/) | Asset V2, deferred rendering, material extensions, automatic batching/instancing, UI materials, and one-shot systems. | Let Bevy batch shared meshes/materials first. Use one-shot systems for genuinely occasional operations, not the main gameplay loop. |
| [0.13](#bevy-013) · [notes](https://bevy.org/news/bevy-0-13/) | Math primitives, automatic deferred-command flush points, system stepping, dynamic queries, camera UI, and light probes. | Build meshes from current primitives, remember commands remain deferred within a system, and use stepping/dynamic queries mainly for tools and diagnostics. |
| [0.14](#bevy-014) · [notes](https://bevy.org/news/bevy-0-14/) | Observers/hooks, explicit color APIs, computed/substates, visibility ranges, animation graphs, and many render effects. | Use observers for reactive events, explicit color spaces, states for modes, and `VisibilityRange` for measured LOD/HLOD. |
| [0.15](#bevy-015) · [notes](https://bevy.org/news/bevy-0-15/) | Required components, picking, gamepads as entities, animation/reflection improvements, BRP, and UI advances. | Prefer direct component spawning backed by required components; query `Gamepad`; use the picking stack rather than hand-built ray plumbing when it fits. |
| [0.16](#bevy-016) · [notes](https://bevy.org/news/bevy-0-16/) | GPU-driven rendering, ECS relationships, unified errors, `no_std` progress, transform optimization, and occlusion culling. | Model hierarchy with relationships, keep errors visible, leave automatic GPU/transform optimizations enabled, and opt into occlusion only after profiling. |
| [0.17](#bevy-017) · [notes](https://bevy.org/news/bevy-0-17/) | Buffered messages were separated from observer events; `On<E>`, `EntityEvent`, headless widgets, web assets, UI gradients, and diagnostic graphs arrived. | Choose messages versus events by semantics, not naming habit. Build app UI from stable widgets; treat Feathers as an evolving tooling-focused layer. |
| [0.18](#bevy-018) · [notes](https://bevy.org/news/bevy-0-18/) | High-level `2d`, `3d`, and `ui` Cargo feature collections, first-party camera controllers, more widgets, and fullscreen materials simplified common work. | Select only the high-level feature collections the product needs; start with first-party controllers/materials before custom infrastructure. |
| [0.19](#bevy-019) · [notes](https://bevy.org/news/bevy-0-19/) | BSN macro scenes, resources as components, system-based render schedules, editable text, a diagnostics overlay, settings, contact shadows, and further GPU/render optimization. | Use the tables below. Note that `.bsn` asset files are **not shipped** in 0.19; use the Rust `bsn!`/`bsn_list!` macros. |

<a id="bevy-01"></a>
### Bevy 0.1: ECS and plugins

- **Definition.** Bevy's original durable idea is that data lives in an ECS world while systems transform that data. Plugins package the systems, schedules, resources, messages, states, and initialization belonging to one capability.
- **Why it matters.** Data-oriented composition avoids deep inheritance trees and makes access visible to Bevy's scheduler. Plugin ownership gives the project a stable architectural seam instead of growing one global `main.rs`.
- **Use it when.** Components are the default for per-entity state; resources are for one value per world; focused plugins are appropriate for every meaningful domain such as movement, combat, cameras, UI, or loading.
- **Think about it as.** A component is a fact, a system is a rule over facts, and a plugin is the module that owns a coherent set of facts and rules. The ECS is not merely a storage API: declared access is what permits safe parallel scheduling.
- **Watch for.** Do not turn every helper into a plugin or every scalar into a component. Choose boundaries that have an invariant, lifecycle, configuration, or reusable behavior to own.

```rust
use bevy::prelude::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, regenerate_health);
    }
}

#[derive(Component, Debug)]
struct Health {
    current: f32,
    maximum: f32,
}

fn regenerate_health(time: Res<Time>, mut health: Query<&mut Health>) {
    let recovered = 4.0 * time.delta_secs();

    for mut health in &mut health {
        let next = (health.current + recovered).min(health.maximum);
        if next != health.current {
            health.current = next;
        }
    }
}
```

This example keeps the rule in the owning plugin and requests only the data it mutates. A real time-based regeneration rule may keep fractional progress in a separate component rather than discarding sub-integer recovery.

<a id="bevy-02"></a>
### Bevy 0.2: task pools, parallel queries, entities, and transforms

- **Definition.** Task pools let Bevy distribute independent work. `Query::par_iter` and `par_iter_mut` expose that capability for sufficiently expensive entity workloads. `Entity` is a generational opaque identifier, while `Transform` and `GlobalTransform` distinguish local from propagated world-space pose.
- **Why it matters.** Generational IDs detect many stale-reference mistakes, and the local/global transform split makes hierarchy composition efficient. Parallel iteration can reduce CPU time without unsafe shared mutation because query access is declared up front.
- **Use it when.** Store an `Entity` when a relationship is temporary or does not deserve a formal ECS relationship. Use `Transform` for local authoring and mutation, `GlobalTransform` for read-only world-space results, and parallel iteration only after profiling shows enough independent work.
- **Think about it as.** `Entity` is a key to validate, never a permanent domain identity or a value whose bit pattern/order has gameplay meaning. `GlobalTransform` is derived output; mutate the authoritative local/domain state instead. Parallel iteration is unordered data-parallel work.
- **Watch for.** Small loops usually lose to task-dispatch overhead. Never depend on parallel/query visit order, worker thread, entity allocation order, or the numeric representation of `Entity`.

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct FollowTarget(Entity);

fn integrate_in_parallel(
    time: Res<Time>,
    mut bodies: Query<(&Velocity, &mut Transform)>,
) {
    let delta_seconds = time.delta_secs();
    bodies.par_iter_mut().for_each(|(velocity, mut transform)| {
        transform.translation += velocity.0 * delta_seconds;
    });
}

fn resolve_targets(
    targets: Query<&GlobalTransform>,
    mut followers: Query<(&FollowTarget, &mut Transform)>,
) {
    for (target, mut follower) in &mut followers {
        let Ok(target_transform) = targets.get(target.0) else {
            continue; // The generational Entity may be stale or not match this query.
        };
        follower.translation = target_transform.translation();
    }
}
```

The first system is valid only if each body's work is independent; benchmark it against ordinary iteration. If following is a stable graph invariant, prefer a custom relationship instead of an unstructured entity reference.

<a id="bevy-03"></a>
### Bevy 0.3: asynchronous assets and scenes

- **Definition.** `AssetServer` returns typed handles immediately and performs loading asynchronously. A glTF scene selected by its asset label can be instantiated through the current `WorldAssetRoot` component.
- **Why it matters.** Disk, decompression, and GPU preparation must not block the main frame loop. Typed handles keep ownership and asset type visible while allowing Bevy to deduplicate and hot-reload supported assets.
- **Use it when.** Use asset handles for images, meshes, materials, fonts, audio, glTF content, and world assets whose lifetime extends beyond a local expression. Put required content behind an explicit loading state when gameplay cannot proceed without it.
- **Think about it as.** A handle is a reference to eventual asset availability, not the asset bytes and not proof that dependencies are ready. The loading screen is part of the product state machine, not an incidental delay.
- **Watch for.** Do not spin, sleep, or synchronously wait on the main thread. Surface missing required assets, retain handles for content that must stay resident, and test cold-cache and failure paths.

```rust
use bevy::prelude::*;

#[derive(Component)]
struct ArenaScene;

fn begin_loading_arena(mut commands: Commands, asset_server: Res<AssetServer>) {
    let arena = asset_server.load("models/arena.glb#Scene0");

    commands.spawn((
        ArenaScene,
        WorldAssetRoot(arena),
        Transform::default(),
    ));
}
```

`WorldAssetRoot` is the Bevy 0.19 name; older examples using `SceneRoot` are migration input. For a strict loading flow, keep the handle in a loading resource and transition state only after the asset and its dependencies report ready.

<a id="bevy-04"></a>
### Bevy 0.4: cross-platform boundaries

- **Definition.** Bevy's web and cross-platform work established a crucial boundary: operating-system and device details belong in adapters, while gameplay rules consume target-neutral data.
- **Why it matters.** A rule coupled directly to one keyboard key, window backend, filesystem convention, or renderer feature is difficult to port, replay, test, or drive from AI/network input. Narrow adapters constrain that cost.
- **Use it when.** Create an adapter whenever keyboard, mouse, touch, gamepad, window, browser, or platform services enter shared game logic. Select target-specific plugins/features only for an explicit supported target.
- **Think about it as.** Hardware produces intent; simulation consumes intent. Target compilation is only the first gate—behavior, assets, input, renderer capability, and performance must also be validated on the target.
- **Watch for.** `cfg` blocks scattered through gameplay are a design smell. A successful desktop build does not establish browser/mobile support, and a working browser renderer does not prove asset or input parity.

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
struct MovementIntent(Vec2);

#[derive(Component)]
struct Player;

fn keyboard_adapter(
    keys: Res<ButtonInput<KeyCode>>,
    mut intent: ResMut<MovementIntent>,
) {
    let mut direction = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    intent.0 = direction.normalize_or_zero();
}

fn move_player(
    intent: Res<MovementIntent>,
    mut player: Single<&mut Transform, With<Player>>,
) {
    player.translation += intent.0.extend(0.0) * 0.1;
}
```

A gamepad, touch, replay, or network adapter can write the same intent without changing `move_player`. For deterministic simulation, buffer bounded intent into `FixedUpdate` and include a fixed delta rather than applying a per-frame constant.

<a id="bevy-05"></a>
### Bevy 0.5: standard physically based rendering

- **Definition.** `StandardMaterial` is Bevy's built-in 3D surface model. It covers ordinary PBR properties such as base color, metallic response, roughness, emissive output, textures, alpha behavior, and an explicit unlit mode.
- **Why it matters.** Using the standard path preserves renderer features, batching opportunities, lighting integration, and future engine improvements. A custom shader is a maintenance boundary and should solve a visual requirement the standard material cannot.
- **Use it when.** Start with `StandardMaterial` for props, characters, terrain, and most environment surfaces. Use unlit for deliberately light-independent visuals; use a custom `Material` or extension for a genuinely different shading model.
- **Think about it as.** Material values describe physical/artistic surface intent. Lighting, environment maps, camera exposure, and tonemapping are separate stages; do not compensate for one stage by arbitrarily distorting another.
- **Watch for.** Share equal material handles to reduce asset churn and improve batching. Mutating a shared material affects every entity using it, so create a distinct asset only when independent variation is intended.

```rust
use bevy::prelude::*;

fn spawn_pbr_object(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let metal = materials.add(StandardMaterial {
        base_color: Color::oklch(0.436_171, 0.094_329, 257.457),
        metallic: 0.85,
        perceptual_roughness: 0.28,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(metal),
        Transform::from_xyz(0.0, 0.0, 0.5),
    ));

    commands.spawn((
        PointLight {
            intensity: 4_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(2.0, -2.0, 4.0),
    ));
}
```

The color is authored in OKLCH, the project's single human-facing color format. Convert it only at a narrow API or shader boundary that explicitly requires a linear representation; lighting math itself remains linear.

<a id="bevy-06"></a>
### Bevy 0.6: the modern renderer, WGSL, and culling

- **Definition.** Bevy separates the main world from extracted render-world data and uses render schedules/systems to prepare, queue, and draw views. `Material` plus `AsBindGroup` is the high-level route for custom WGSL surface data; automatic frustum culling rejects off-camera bounded geometry.
- **Why it matters.** The split lets simulation continue independently of pipelined rendering and gives the renderer a purpose-built representation. High-level materials reuse Bevy's mesh pipeline instead of forcing every visual effect into low-level render code.
- **Use it when.** Prefer `StandardMaterial`, then a material extension, then a custom `Material`. Add a custom render-schedule system only when the effect is a pass rather than a surface and profiling/design rules justify the complexity.
- **Think about it as.** Extraction is a synchronization boundary: changing an asset/component may create render-world work. Culling is the default visibility budget, so accurate bounds and normal visibility components are part of performance correctness.
- **Watch for.** Avoid `NoFrustumCulling` unless geometry truly cannot be represented by bounds. Keep bind-group layouts and shader variants small, and validate shaders on every selected renderer backend.

```rust
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

const SHADER_PATH: &str = "shaders/heat_material.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct HeatMaterial {
    #[uniform(0)]
    color: LinearRgba,
    #[uniform(1)]
    intensity: f32,
}

impl Material for HeatMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }
}

fn install_material(app: &mut App) {
    app.add_plugins(MaterialPlugin::<HeatMaterial>::default());
}
```

The WGSL shader must declare bindings matching the derived layout. Register the material plugin once, store `Handle<HeatMaterial>` inside `MeshMaterial3d<HeatMaterial>`, and let the normal mesh pipeline handle view selection and culling. `LinearRgba` is the required GPU-boundary representation here: author the source as `Color::oklch(...)` and call `.to_linear()` only when constructing the material.

<a id="bevy-07"></a>
### Bevy 0.7: specialized workloads and `ParamSet`

- **Definition.** This release broadened the engine with animation, compressed textures, compute, render-to-texture, multi-entity access, and `ParamSet`. The common principle is specialization: each tool addresses a workload that ordinary sequential ECS or standard rendering cannot express cleanly.
- **Why it matters.** `ParamSet` preserves normal system parameters and scheduler validation when one system needs accesses that would otherwise conflict. Compressed textures reduce bandwidth/residency; compute and render targets move suitable work/data flows onto the GPU.
- **Use it when.** Use `ParamSet` for sequential phases within one cohesive rule. Use texture compression for production asset budgets, animation for authored temporal state, and GPU/target features only when their data flow is naturally parallel or visual.
- **Think about it as.** Specialized APIs spend complexity to buy a concrete capability or budget. First prove the simple path is inadequate, then isolate the specialized path behind a narrow plugin or material/render boundary.
- **Watch for.** `ParamSet` is not permission for arbitrary broad queries; model disjoint access with filters when that is true. GPU work can add synchronization and portability costs, and compressed formats must be validated on selected targets.

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct FollowCamera;

fn place_follow_camera(
    mut transforms: ParamSet<(
        Query<&mut Transform, With<FollowCamera>>,
        Query<&Transform, With<Player>>,
    )>,
) {
    // Read phase: copy the value before borrowing the other query.
    let player_position = {
        let players = transforms.p1();
        let Ok(player) = players.single() else {
            return;
        };
        player.translation
    };

    // Write phase: ParamSet guarantees only one conflicting query is active.
    for mut camera in &mut transforms.p0() {
        camera.translation = player_position + Vec3::new(0.0, -6.0, 4.0);
        camera.look_at(player_position, Vec3::Z);
    }
}
```

If the camera and player filters are a guaranteed disjoint model, disjoint `With`/`Without` filters may be clearer and allow simultaneous parameters. If these phases become independently useful, split them into systems and order only the real dependency.

<a id="bevy-08"></a>
### Bevy 0.8: camera-driven views, inherited visibility, and modern UI roots

- **Definition.** Cameras define views; renderable entities participate according to visibility and layer/filter rules. A child's effective visibility combines its local `Visibility` with ancestor state through `InheritedVisibility`. UI layout is component data evaluated by the Taffy-backed layout engine.
- **Why it matters.** View-centric rendering supports multiple cameras, render targets, and UI targeting without embedding a single global camera assumption. Inherited visibility lets an entire actor or UI panel be toggled at one stable root.
- **Use it when.** Put a coherent visual subtree beneath an entity when transform and visibility should propagate together. Use `Node` for declarative UI layout, and add cameras intentionally for world, minimap, split-screen, or render-target views.
- **Think about it as.** `Visibility` is author/game intent, `InheritedVisibility` is the hierarchy result, and final view visibility is renderer-computed. UI layout is a tree constraint problem, so stable hierarchy and localized updates matter.
- **Watch for.** Do not write `InheritedVisibility` directly or manually synchronize every child. Multiple cameras multiply visible-work and post-processing costs; target cameras/layers explicitly so views do not render accidental content.

```rust
use bevy::prelude::*;

#[derive(Component)]
struct ActorVisualRoot;

fn spawn_hidden_actor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Capsule3d::default());
    let material = materials.add(Color::oklch(0.732_032, 0.153_756, 240.894));

    commands.spawn((
        ActorVisualRoot,
        Visibility::Hidden,
        Transform::default(),
        children![(
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, 0.0, 0.5),
        )],
    ));
}

fn reveal_actor(mut roots: Query<&mut Visibility, With<ActorVisualRoot>>) {
    for mut visibility in &mut roots {
        *visibility = Visibility::Inherited;
    }
}
```

The child keeps its local visibility inherited while the root controls the subtree. The same pattern works for a UI panel authored with `Node` children; mutate the stable root instead of rebuilding the whole tree.

<a id="bevy-09"></a>
### Bevy 0.9: HDR, tonemapping, bloom, and camera effects

- **Definition.** HDR retains scene luminance beyond display range, tonemapping maps that range to the output display, bloom spreads selected bright energy, and anti-aliasing reduces edge artifacts. These are view/camera presentation decisions.
- **Why it matters.** Lighting and emissive art can preserve meaningful intensity before the display transform. Per-camera components make the visual pipeline explicit and allow different views to carry different quality/effect budgets.
- **Use it when.** Enable HDR when the lighting/post stack requires it. Choose a tonemapper as an art-direction decision, add bloom when bright sources should glow, and select anti-aliasing from measured quality/performance on target hardware.
- **Think about it as.** The order is roughly scene lighting into an HDR image, optional post effects, then a display transform. An effect is not free merely because it is a component; most costs scale with resolution and number of views.
- **Watch for.** Do not stack every effect by default. Compare representative bright/dark scenes, motion, transparency, and UI; measure combined GPU cost because effects can interact and allocate intermediate textures.

```rust
use bevy::{
    camera::Hdr,
    core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::Bloom,
    prelude::*,
};

fn spawn_cinematic_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Tonemapping::TonyMcMapface,
        Bloom::default(),
        Transform::from_xyz(0.0, -8.0, 5.0)
            .looking_at(Vec3::ZERO, Vec3::Z),
    ));
}
```

Treat this as an explicit quality preset, not an unconditional camera recipe. Benchmark with the intended output resolution and worst-case view; disable or tune individual effects to meet the frame budget.

<a id="bevy-010"></a>
### Bevy 0.10: schedule contracts and windows as entities

- **Definition.** Modern Bevy schedules form a dependency graph of systems. System sets name stable phases; `.before`, `.after`, and `.chain()` add genuine edges. Windows are ECS entities with `Window` components rather than one implicit global object.
- **Why it matters.** Explicit dependency edges preserve parallel execution everywhere else and make cross-plugin contracts reviewable. Entity-based windows allow multiple windows and ordinary filtered query access.
- **Use it when.** Define sets when several systems share a phase or another plugin must order against it. Query `With<PrimaryWindow>` for the main window, or keep a selected window `Entity` when multi-window behavior needs identity.
- **Think about it as.** Ordering is a partial order, not a master list. Add the minimum edges required by data/semantic dependencies; scheduler order between independent systems should remain unspecified.
- **Watch for.** Plugin registration order, query iteration, and entity ID order are not gameplay sequencing. Chaining an entire frame serializes independent work, and assuming exactly one window without a filter makes tools/multi-window support brittle.

```rust
use bevy::{prelude::*, window::PrimaryWindow};

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FramePhase {
    Input,
    Simulation,
    Presentation,
}

fn install_schedule(app: &mut App) {
    app.configure_sets(
        Update,
        (
            FramePhase::Input,
            FramePhase::Simulation,
            FramePhase::Presentation,
        )
            .chain(),
    )
    .add_systems(Update, read_input.in_set(FramePhase::Input))
    .add_systems(Update, simulate.in_set(FramePhase::Simulation))
    .add_systems(Update, update_window.in_set(FramePhase::Presentation));
}

fn update_window(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    let desired_title = "Arenic";
    if window.title != desired_title {
        window.title = desired_title.into();
    }
}

fn read_input() {}
fn simulate() {}
```

This short chain documents a real frame pipeline. Systems within the same phase remain unordered unless their own access or explicit constraints require otherwise.

<a id="bevy-011"></a>
### Bevy 0.11: typed schedules and fixed simulation

- **Definition.** `add_systems(Schedule, systems)` makes the execution context explicit. `Update` follows rendered frames; `FixedUpdate` can run zero, one, or multiple fixed ticks to catch up. Gizmos and ECS audio make diagnostics and playback ordinary scheduled/entity behavior.
- **Why it matters.** Fixed ticks produce stable rule cadence for movement, cooldowns, replay, and networking, while variable-rate input/presentation stays responsive. Typed schedules make ownership and timing visible at registration.
- **Use it when.** Put device reads, UI, cameras, and presentation in `Update`; put rule-critical fixed-step simulation in `FixedUpdate`. Use gizmos for debug views and `AudioPlayer` entities for playback lifecycles.
- **Think about it as.** Input and simulation are different clocks. `Update` samples hardware into bounded intent; `FixedUpdate` consumes an explicitly defined amount. Presentation then renders the latest state or an interpolated view.
- **Watch for.** A frame may produce multiple fixed ticks, so unbounded or repeatedly reused input can amplify actions. Do not assume one `FixedUpdate` per frame, and do not put renderer/device APIs into deterministic rule logic.

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Simulated;

#[derive(Component)]
struct Velocity(Vec3);

fn install_simulation(app: &mut App) {
    app.add_systems(FixedUpdate, integrate_velocity)
        .add_systems(Update, draw_velocity_debug);
}

fn integrate_velocity(
    time: Res<Time<Fixed>>,
    mut bodies: Query<(&Velocity, &mut Transform), With<Simulated>>,
) {
    for (velocity, mut transform) in &mut bodies {
        transform.translation += velocity.0 * time.delta_secs();
    }
}

fn draw_velocity_debug(
    mut gizmos: Gizmos,
    bodies: Query<(&Velocity, &Transform), With<Simulated>>,
) {
    for (velocity, transform) in &bodies {
        gizmos.line(
            transform.translation,
            transform.translation + velocity.0,
            Color::oklch(0.865_623, 0.173_053, 89.879),
        );
    }
}
```

The gizmo is a presentation-only diagnostic of authoritative simulation data. If rendered motion needs to be smooth between fixed states, maintain a presentation transform/interpolation layer rather than changing the fixed rule cadence.

<a id="bevy-012"></a>
### Bevy 0.12: Asset V2, automatic batching, and renderer extensions

- **Definition.** Bevy's asset system owns typed assets and dependency-aware loading. Compatible entities sharing mesh/material pipeline state can be batched or instanced automatically; material extensions customize the standard pipeline without replacing it.
- **Why it matters.** Asset sharing reduces memory, uploads, and state changes. Automatic batching captures common performance wins while preserving ordinary ECS entities, and extensions retain built-in lighting/shadow behavior.
- **Use it when.** Load or create reusable meshes/materials once, retain their handles in an owning resource, and clone handles into typed components. Reach for a material extension when a small shader addition should coexist with `StandardMaterial`.
- **Think about it as.** The asset is shared immutable-ish data; the handle is cheap identity; the entity carries per-instance state. The renderer can batch only entities whose relevant pipeline/material state is compatible.
- **Watch for.** Creating an equal material per entity defeats sharing. Mutating one shared asset changes all users and may cause extraction/upload work; use per-instance components or intentional distinct assets for actual variation.

```rust
use bevy::prelude::*;

#[derive(Resource)]
struct SharedUnitVisual {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

fn prepare_unit_visual(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(SharedUnitVisual {
        mesh: meshes.add(Capsule3d::default()),
        material: materials.add(Color::oklch(0.751_225, 0.181_076, 147.289)),
    });
}

fn spawn_units(mut commands: Commands, visual: Res<SharedUnitVisual>) {
    for x in -50..50 {
        commands.spawn((
            Mesh3d(visual.mesh.clone()),
            MeshMaterial3d(visual.material.clone()),
            Transform::from_xyz(x as f32, 0.0, 0.5),
        ));
    }
}

fn install_units(app: &mut App) {
    app.add_systems(Startup, (prepare_unit_visual, spawn_units).chain());
}
```

The ordering edge allows the inserted resource to be available before the spawn system, and every unit shares the same asset identities. Profile before replacing this ordinary ECS path with custom instancing.

<a id="bevy-013"></a>
### Bevy 0.13: primitives and deferred-command boundaries

- **Definition.** Current math primitives such as `Circle`, `Cuboid`, `Sphere`, `Cone`, and `Capsule3d` are reusable geometric descriptions that can become meshes or power non-render math. `Commands` queues structural ECS changes, and the scheduler inserts deferred application at valid ordered boundaries.
- **Why it matters.** One primitive vocabulary reduces mismatches between visualization and rule geometry. Deferred structural changes allow systems to iterate stable archetype storage while requesting spawns, despawns, insertions, and removals safely.
- **Use it when.** Use primitives for standard procedural shapes and their geometric operations. Use `Commands` for structural changes; use direct query/resource mutation when an existing value must change immediately.
- **Think about it as.** A command is a request against the world after the current system, not an immediate mutation visible inside that system. A declared dependency can create the flush boundary needed by a later system.
- **Watch for.** Never assume a commanded entity/component is query-visible immediately. Do not add arbitrary global flushes; order the producer and consumer only when the consumer truly needs the structural result.

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Enemy;

fn spawn_enemy_once(
    mut commands: Commands,
    mut spawned: Local<bool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *spawned {
        return;
    }
    *spawned = true;

    commands.spawn((
        Enemy,
        Mesh3d(meshes.add(Cone::new(0.5, 1.0))),
        MeshMaterial3d(materials.add(Color::oklch(0.595_752, 0.222_113, 24.719))),
        Transform::default(),
    ));
}

fn place_new_enemies(mut enemies: Query<&mut Transform, Added<Enemy>>) {
    for mut transform in &mut enemies {
        transform.translation = Vec3::new(3.0, 2.0, 0.5);
    }
}

fn install_enemy_setup(app: &mut App) {
    app.add_systems(Update, (spawn_enemy_once, place_new_enemies).chain());
}
```

The chain expresses that `place_new_enemies` must observe the commanded spawn; Bevy applies deferred commands at the dependency boundary. If placement is known at spawn time, putting the final `Transform` in the spawn tuple is simpler and avoids the second system.

<a id="bevy-014"></a>
### Bevy 0.14: observers, explicit color, states, and visibility ranges

- **Definition.** Observers react immediately when an `Event` or entity-targeted `EntityEvent` is triggered. OKLCH is the project's color-authoring space, with explicit conversion at renderer boundaries. States model application modes, while `VisibilityRange` supports distance-driven LOD/HLOD visibility.
- **Why it matters.** Observers put reactive behavior at the event boundary without polling every frame. Explicit color space prevents subtle lighting/blending errors; states let the scheduler skip whole inactive domains; range visibility reduces distant rendering cost.
- **Use it when.** Use observers for damage reactions, clicks, lifecycle behavior, or propagated entity events. Use states for mutually meaningful modes and `VisibilityRange` only with authored distance representations and measured need.
- **Think about it as.** An event is a synchronous reaction, not buffered history. State is scheduler topology, not just a boolean. Color space is a data contract, and LOD is a content-plus-runtime system rather than one magic component.
- **Watch for.** Keep observers bounded and avoid trigger cycles. Use messages when consumers should read buffered data later. Convert authored OKLCH colors before linear lighting math, and do not add range thresholds without testing popping across camera FOV/speed.

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Health(u16);

#[derive(EntityEvent)]
struct Damage {
    entity: Entity,
    amount: u16,
}

fn apply_damage(damage: On<Damage>, mut health: Query<&mut Health>) {
    let Ok(mut health) = health.get_mut(damage.entity) else {
        return;
    };
    health.0 = health.0.saturating_sub(damage.amount);
}

fn damage_target(mut commands: Commands, target: Single<Entity, With<Health>>) {
    commands.trigger(Damage {
        entity: *target,
        amount: 10,
    });
}

fn install_damage_observer(app: &mut App) {
    app.add_observer(apply_damage);
}
```

`Damage` targets one entity, so the entity field is part of the event contract and is available through `On<Damage>`. A periodic stream, input history, or cross-schedule backlog belongs in a `Message` instead.

<a id="bevy-015"></a>
### Bevy 0.15: required components, picking, and gamepads as entities

- **Definition.** Required components declare flat insertion invariants: adding one component automatically supplies its requirements. Gamepads are entities carrying `Gamepad`, and Bevy's picking stack provides common pointer hit/interaction infrastructure.
- **Why it matters.** Required components eliminate repeated construction bundles while making invalid component combinations harder to create. Entity gamepads fit normal ECS lifecycle/query patterns; picking avoids duplicating camera rays, hit ordering, focus, and event plumbing.
- **Use it when.** Use requirements when component `A` is never valid without flat components `B`/`C`. Query gamepads in hardware adapters. Use picking for pointer-driven UI/world interactions unless the game's hit semantics are genuinely specialized.
- **Think about it as.** A requirement is a local type invariant, not a prefab. Gamepad entities are devices that appear/disappear. Picking produces interaction facts; domain rules should still decide what a click means.
- **Watch for.** Do not hide large mutable actor compositions behind transitive requirements—use scenes for hierarchical/dependency-rich construction. Do not encode controller-specific rules in gameplay or assume one connected gamepad.

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Health(u16);

#[derive(Component)]
#[require(Health(100), Transform)]
struct ControllableHero;

fn spawn_hero(mut commands: Commands) {
    // Health(100) and Transform::default() are inserted automatically.
    commands.spawn(ControllableHero);
}

fn read_join_buttons(gamepads: Query<(Entity, &Gamepad)>) {
    for (device, gamepad) in &gamepads {
        if gamepad.just_pressed(GamepadButton::South) {
            info!(?device, "gamepad requested to join");
        }
    }
}
```

If the hero also needs a mesh hierarchy, weapon sockets, and asset dependencies, keep this flat rule invariant and compose the visual/child structure with BSN or another scene boundary.

<a id="bevy-016"></a>
### Bevy 0.16: ECS relationships and automatic GPU/transform work

- **Definition.** Relationships model directed ECS edges with a source component and a maintained target collection. `ChildOf`/`Children` is the hierarchy relationship. Bevy also gained broader GPU-driven rendering, static-transform optimization, and opt-in GPU occlusion culling.
- **Why it matters.** Relationships make graphs queryable and keep inverse collections consistent. Automatic renderer/transform paths can skip or move large workloads without replacing ordinary entities with custom infrastructure.
- **Use it when.** Use `ChildOf` for transform/visibility hierarchy and custom relationships for stable domain graphs such as ownership or equipment. Leave static transform optimization/GPU-driven rendering enabled; test occlusion culling for dense scenes with meaningful occluders.
- **Think about it as.** The source component is authoritative and the target collection is derived. Automatic optimizations are the baseline; specialization should follow a measured bottleneck rather than precede it.
- **Watch for.** Never mutate `Children` directly. Deep/churning hierarchies have propagation cost. Occlusion requires `DepthPrepass`, is disabled by default, and in Bevy 0.19 is incompatible with `DeferredPrepass`.

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Hero;

#[derive(Component)]
struct ShoulderCamera;

fn attach_shoulder_camera(mut commands: Commands) {
    let hero = commands
        .spawn((Hero, Transform::default()))
        .id();

    commands.spawn((
        ShoulderCamera,
        Camera3d::default(),
        ChildOf(hero),
        Transform::from_xyz(0.8, -3.0, 1.2)
            .looking_at(Vec3::new(0.8, 4.0, 0.0), Vec3::Z),
    ));
}

fn inspect_parent(cameras: Query<&ChildOf, With<ShoulderCamera>>) {
    for child_of in &cameras {
        let hero = child_of.parent();
        info!(?hero, "camera follows this hero");
    }
}
```

Despawning the hero follows the current linked-child behavior and despawns its child camera. For a reference that must survive target despawn or should not imply transform/ownership propagation, model a different relationship or validated `Entity` reference.

<a id="bevy-017"></a>
### Bevy 0.17: messages versus observer events

- **Definition.** `Message` is buffered communication read later through per-reader cursors. `Event`/`EntityEvent` is triggered behavior observed immediately through `On<E>`. Headless widgets similarly separate interaction semantics from visual skin.
- **Why it matters.** Naming the two communication semantics prevents buffered gameplay streams from being confused with synchronous reactions. Multiple message readers retain independent positions, while observers can target and propagate around an entity graph.
- **Use it when.** Use messages for input intent, decoupled producers/consumers, and schedule boundaries. Use direct mutation for local rules. Use events for immediate reactions, clicks, lifecycle behavior, and entity-targeted propagation.
- **Think about it as.** A message says “this data is available to readers”; an event says “react now.” A widget says “this interaction/state contract exists,” while its node/text/material components decide appearance.
- **Watch for.** Bound message production and consumption, define overflow/coalescing behavior, and register message types. Keep observers bounded and cycle-free. Do not mechanically rename old events—choose semantics first.

```rust
use bevy::prelude::*;

#[derive(Message)]
struct MoveIntent(Vec2);

#[derive(EntityEvent)]
struct Damage {
    entity: Entity,
    amount: u16,
}

fn write_movement(mut moves: MessageWriter<MoveIntent>) {
    moves.write(MoveIntent(Vec2::X));
}

fn consume_movement(mut moves: MessageReader<MoveIntent>) {
    let mut chosen = None;
    for movement in moves.read() {
        chosen.get_or_insert(movement.0);
    }
    if let Some(direction) = chosen {
        info!(?direction, "consume one coalesced movement intent");
    }
}

fn observe_damage(damage: On<Damage>) {
    info!(target = ?damage.entity, amount = damage.amount, "react now");
}

fn install_communication(app: &mut App) {
    app.add_message::<MoveIntent>()
        .add_observer(observe_damage)
        .add_systems(Update, (write_movement, consume_movement).chain());
}
```

The chain documents that this example's reader consumes after the writer. In a real input-to-fixed-simulation design, define how intent survives/collapses across the `Update` and `FixedUpdate` clocks.

<a id="bevy-018"></a>
### Bevy 0.18: high-level feature collections and first-party controllers

- **Definition.** Bevy's `2d`, `3d`, `ui`, and later-independent `audio` Cargo feature collections group coherent subsystems. First-party `PanCamera` and `FreeCamera` controllers provide reusable camera behavior through plugins/components.
- **Why it matters.** High-level features are more stable and understandable than manually reconstructing a large low-level feature graph. Controllers provide a tested baseline for editors, debug views, map navigation, and scene exploration.
- **Use it when.** With `default-features = false`, select the product's high-level collections explicitly. Use `PanCamera` for conventional 2D pan/zoom and `FreeCamera` for 3D exploration/debug tooling; build a domain camera when game-specific constraints dominate.
- **Think about it as.** Cargo features define compiled capability, while plugins/components activate runtime behavior. A controller is an adapter/presentation tool, not authoritative gameplay state.
- **Watch for.** In Bevy 0.19, `2d`/`3d` do not imply `ui`, and none imply `audio`. Controller defaults encode input conventions, so review bindings, enablement, focus/cursor behavior, and target suitability.

```rust
use bevy::{
    camera_controller::pan_camera::{PanCamera, PanCameraPlugin},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanCameraPlugin)
        .add_systems(Startup, spawn_map_camera)
        .run();
}

fn spawn_map_camera(mut commands: Commands) {
    commands.spawn((Camera2d, PanCamera::default()));
}
```

This is appropriate for a map/editor/debug view. A shipped tactical camera with edge scrolling, bounds, focus targets, gamepad semantics, and replay requirements likely deserves a focused game-owned plugin instead.

<a id="bevy-019"></a>
### Bevy 0.19: BSN scenes and the current architecture

- **Definition.** Bevy Scene Notation (`bsn!`, `bsn_list!`) is a Rust macro grammar for declarative scene composition, template patching, children, asset dependencies, entity references, and observers. `SceneComponent` associates a component with a scene. Bevy 0.19 also makes resources components internally and replaces camera render-graph authoring with render-schedule systems.
- **Why it matters.** BSN makes complex entity composition compact and reviewable while retaining Rust functions, types, expressions, and refactoring tools. Scene components can make a component plus its hierarchical presentation one declarative invariant.
- **Use it when.** Use BSN for reusable actors, nested UI, visual rigs, startup worlds, and asset-rich hierarchies. Use direct component tuples for small/dynamic local construction, and required components for flat high-frequency invariants.
- **Think about it as.** A scene describes what to build; systems describe how it behaves over time. Compose small scene functions rather than building one giant macro. Treat render schedules and resources-as-components as engine architecture details that make query scope/access precision more important.
- **Watch for.** Bevy 0.19 does not ship the `.bsn` asset-file loader. `SceneComponent` must be spawned through scene APIs. Broad queries can include resource entities, so filter by domain or `Without<IsResource>` where ordinary entities are intended.

```rust
use bevy::prelude::*;

#[derive(Component, Default, Clone)]
struct Sword;

#[derive(SceneComponent, Default, Clone)]
struct HeroScene {
    health: u16,
}

impl HeroScene {
    fn scene() -> impl Scene {
        bsn! {
            #HeroVisual
            Children [
                #RightHand Sword,
            ]
        }
    }
}

fn spawn_hero(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        @HeroScene { health: 100 }
    });
}
```

`@HeroScene` inserts the component and resolves its associated scene; `#HeroVisual`/`#RightHand` create local names and entity references. If the scene declares unresolved asset dependencies, use `queue_spawn_scene`. Keep movement, combat, input, and other evolving rules in scheduled systems/messages/observers.

## App structure and scheduling

| Feature or syntax | Recommendation | When to use it | How to use it correctly |
|---|---|---|---|
| Focused plugins | **Default** | Every gameplay or presentation domain. | Give each plugin ownership of related initialization, systems, messages/events, resources, states, and schedules. Keep `main` declarative. |
| Direct component tuples | **Default** | A small entity assembled close to its dynamic inputs. | Spawn `(Marker, Transform, Mesh3d(...), MeshMaterial3d(...))`. Do not resurrect removed engine bundle structs. |
| Required components | **Default** for flat invariants | One component always requires other components to be present. | Declare requirements on the owning component so every insertion maintains the invariant. Prefer this over repeating construction boilerplate. |
| Custom `#[derive(Bundle)]` | **Use when** | A stable, meaningful component group is reused broadly and required components or a scene would be less clear. | Give the bundle a domain name and document its invariant. Bundles still exist; old engine convenience bundles do not. |
| BSN scene functions | **Use when** | Reusable declarative entities, nested UI, actor presentation, asset references, named cross-references, or reviewable composition. | Return `impl Scene`; compose with `bsn!`; derive `Default + Clone` or `FromTemplate` for participating components. Keep dynamic rule logic in systems. |
| `bsn_list!` | **Use when** | A startup arena, level slice, or sibling group contains multiple roots. | Return `impl SceneList`; separate root entities with commas; compose scene functions rather than one giant macro. |
| `SceneComponent` | **Use when** | A component should declaratively provide a hierarchical or dependency-aware subscene. | Derive `SceneComponent, Default, Clone`, implement `Self::scene() -> impl Scene`, and include it as `@Type { ... }` in BSN. It must be spawned through scene APIs. Prefer required components for flat, hot-spawn invariants. |
| `Startup` | **Default** for one-time setup | Cameras, initial world, startup resources that depend on the world. | `add_systems(Startup, setup)`. Use plugin/resource initialization APIs when no system is needed. |
| `Update` | **Default** for input and presentation | Device input, UI, camera presentation, effects, and variable-rate work. | Keep hardware-specific reads here and translate them to device-neutral intent. |
| `FixedUpdate` | **Default** for deterministic ticks | Physics-like rules, replay/network-relevant simulation, or fixed-rate gameplay. | Buffer bounded intent from `Update`, consume it in `FixedUpdate`, and keep rendering interpolation/presentation separate. |
| System sets | **Default** for cross-plugin phase contracts | Several systems share a meaningful phase or other plugins must order against it. | Define a `SystemSet`, configure it once, and order only genuine dependencies. |
| `.chain()` | **Use when** | Every adjacent system has a real data/deferred-command dependency. | Chain a short pipeline. Do not chain unrelated systems; it removes scheduler parallelism. |
| `.before()` / `.after()` | **Use when** | A precise pair or set dependency exists. | Order against stable sets when possible, not implementation-detail system names across domains. |
| Run conditions | **Default** for stable inactivity | A system should sleep when its state/message/resource condition is false. | Use `run_if`, state-scoped scheduling, or a specific common condition. Avoid waking every frame merely to return. |
| `Commands` | **Default** for structural changes | Spawn, despawn, insert/remove components, add observers, or change relationships. | Treat effects as deferred. Insert/mutate through a query when the existing entity must change immediately. Add explicit ordering only when a later system must observe the structural change. |
| Exclusive systems | **Use when** | The operation genuinely needs arbitrary `World` access. | Keep them narrow and infrequent; normal systems preserve scheduling parallelism and validate access better. |
| One-shot systems | **Use when** | An occasional operation benefits from system parameters but should not run on a schedule. | Register/run deliberately. Do not use them to hide recurring gameplay behavior from the scheduler. |
| Bevy states | **Use when** | The app has explicit modes such as loading, menu, playing, paused, or game over. | Define state types and schedule entry/exit/in-state work. Do not replace unrelated state with a bag of booleans. |
| Render schedules | **Use when** | Implementing a custom camera render pass. | Add a render-world system to `Core2d` or `Core3d`, query views through `ViewQuery`, and constrain it with render sets. The old camera `RenderGraph` node API is removed. |

## ECS data, queries, and communication

| Feature or syntax | Recommendation | When to use it | How to use it correctly |
|---|---|---|---|
| Components | **Default** for per-entity state | Many entities may have independent values. | Keep components cohesive and queries narrow. Separate authoritative rules from render-only state when determinism matters. |
| Marker components | **Default** for binary traits | Presence means true and absence means false. | Use a zero-sized marker such as `Selected`; avoid `selected: bool` unless false has meaning distinct from absence. |
| Resources | **Default** for unique global state/services | Exactly one value exists for the world. | `#[derive(Resource)]` now also implements `Component`; do not also derive `Component`, and do not reuse the same type as ordinary entity state. |
| `Local<T>` | **Use when** | One system needs private persistent state. | Use for caches/counters owned by that system, not state other systems must inspect or coordinate around. |
| Direct query mutation | **Default** for local communication | Producer and consumer are the same rule boundary or the target already exists. | Mutate the relevant component/resource directly. Do not introduce a queue merely to move data across one function-sized boundary. |
| Messages | **Default** for buffered many-to-many communication | Input intent, decoupled producers/consumers, or schedule boundaries. | Derive `Message`, call `add_message::<T>()`, write with `MessageWriter::write`, read with `MessageReader::read`. Bound production and consumption. |
| Events and observers | **Default** for reactive behavior | Immediate trigger/response, global observation, or entity-targeted reaction. | Derive `Event` or `EntityEvent`; observer systems take `On<E>`. Use propagation only when relationship semantics require it. |
| Lifecycle hooks/events | **Use when** | A component must react to insertion/removal/despawn as part of its invariant. | Use current `Add`, `Insert`, `Discard`, `Remove`, and `Despawn` semantics. Keep hooks lightweight and documented. |
| `Query::single()` / `single_mut()` | **Use when** | Exactly one entity is expected, but failure should be handled locally. | Match or propagate the returned `Result`; include filters that express the invariant. |
| `Single<...>` system parameter | **Use when** | Exactly-one cardinality is a real precondition for running the system. | Use a precise filtered `Single`. Consider a run condition or alternative query if absence is routine rather than exceptional. |
| Narrow queries | **Default** | Always. | Request only needed components; request `&mut T` only when mutating. Filter to domain markers so broad queries do not accidentally include resource entities. |
| Broad entity queries | **Avoid** by default | Tooling or intentionally world-wide inspection only. | `Query<Entity>`, `EntityRef`, and `Option<&T>` can see abstract resource entities in 0.19. Narrow by domain marker or use `Without<IsResource>` where appropriate. |
| `Added<T>` / `Changed<T>` | **Use when** | Bevy change-detection semantics match the rule. | Remember mutable dereference can mark a value changed even if the final value is equal. Compare before mutation when downstream work is expensive. |
| `ParamSet` | **Use when** | Legitimate system parameters have conflicting accesses that occur in separate phases. | Put the conflicting queries in a `ParamSet` and access one at a time. Do not use it to mask a poor data model. |
| `Query::get_many` / combinations | **Use when** | A bounded set of entities or pairs must be processed. | Handle missing/aliasing errors explicitly. Avoid all-pairs work unless entity counts are strictly bounded. |
| Parallel query iteration | **Measure first** | Each entity does enough independent CPU work and entity count is high. | Use `par_iter()`/`par_iter_mut().for_each(...)`; never rely on visit order or worker thread. Tune batch size only from measurements. |
| Table component storage | **Default** | Most components, especially frequently iterated ones. | Let `#[derive(Component)]` use the default table storage. |
| Sparse-set storage | **Measure first** | Insertion/removal churn dominates and iteration locality is less important. | Use `#[component(storage = "SparseSet")]` only for a measured high-churn component. Benchmark the real workload. |
| Relationships | **Default** for hierarchy/graphs | Parent-child, ownership, equipment, following, or domain graphs. | Use a relationship source such as `ChildOf`; let Bevy maintain the target collection such as `Children`. Never mutate `Children` directly. |
| Recursive despawn | **Default behavior** | Removing an entity with linked `Children`. | `despawn()` follows the current linked-spawn relationship behavior; do not call removed `despawn_recursive`. Confirm other custom relationship behavior explicitly. |
| Deterministic iteration | **Use when** | Outcomes, replays, lockstep, or tests depend on order. | Do not use query iteration or entity IDs as rule ordering. Collect stable domain IDs and sort, or encode order directly in deterministic state. |

## Input, assets, scenes, and platform boundaries

| Feature or syntax | Recommendation | When to use it | How to use it correctly |
|---|---|---|---|
| `ButtonInput<KeyCode>` / `ButtonInput<MouseButton>` | **Default** | Keyboard or mouse adapter systems. | Choose `pressed` versus `just_pressed` deliberately and translate device input into shared gameplay intent. |
| `Query<&Gamepad>` | **Default** | Controller adapter systems. | Query `Gamepad` components and inspect buttons/axes there. Do not use the obsolete `Gamepads` resource. |
| Device-neutral intent | **Default** | Gameplay can be controlled by more than one device or replayed. | Keep device rules in adapters; send a bounded message or mutate an intent resource/component consumed by gameplay. |
| Bounded input | **Default** | Fixed-tick movement, commands, and network/replay paths. | Define how many inputs can be consumed per tick, how repeats work, and what happens on overflow. |
| `AssetServer::load` | **Default** | Runtime assets loaded by path. | Store the typed handle in an owning resource/component or current typed presentation component. Loading is asynchronous; expose missing required assets. |
| `load_builder` | **Use when** | Per-load settings or advanced asset configuration are required. | Keep configuration near the owning asset boundary and document why normal `load` is insufficient. |
| Shared handles | **Default** | Many entities use the same mesh, material, image, font, or scene. | Load/add once, clone the handle, and avoid creating semantically identical assets per entity. This improves memory use and batching. |
| `Assets<T>::get_mut` / `AssetMut` | **Use when** | A shared or unique asset truly changes. | Compare before mutation. In 0.19 a `Modified` event is emitted only when `AssetMut` is actually mutably dereferenced, so avoid needless mutation and render re-extraction. |
| BSN macros | **Use when** | Code-authored reusable scene composition. | Use `bsn!` for one scene and `bsn_list!` for sibling roots. `Commands::spawn_scene` resolves now; `queue_spawn_scene` waits for declared dependencies. |
| `.bsn` files | **Avoid in core Bevy 0.19** | None without a separately evaluated ecosystem loader. | Bevy 0.19 ships the macro syntax but not the `.bsn` asset loader. Keep production scenes in Rust macros for now. |
| Scene functions | **Default** for reusable BSN | Actor variants or parameterized presentation. | Return `impl Scene`, accept narrow parameters, and compose smaller functions. Keep the macro readable rather than embedding complex control flow. |
| glTF scene root | **Default** | Instantiating a loaded glTF scene/world asset. | Attach `WorldAssetRoot(asset_server.load("model.glb#Scene0"))`. `SceneRoot` is the pre-0.19 name. |
| Typed asset components | **Default** | Attaching render/scene/audio assets to entities. | Use `Mesh3d`, `Mesh2d`, material wrappers, `WorldAssetRoot`, `DynamicWorldRoot`, and `AudioPlayer`; a bare `Handle<T>` is not an entity component. |
| High-level Cargo features | **Default** | Selecting Bevy subsystems. | Plain `bevy = "0.19"` enables `2d`, `3d`, `ui`, and `audio`. With defaults disabled, select them deliberately: `2d`/`3d` do not imply `ui`, and none of them imply `audio`. |
| `default-features = false` | **Use when** | Build time, binary size, platform constraints, or dependency surface justify explicit selection. | Start with the high-level collections the product needs, then validate every selected target and asset type. Avoid fragile lists of low-level internals without a concrete need. |
| Target-specific integration | **Use when** | A selected platform truly needs different windowing, input, renderer, or asset behavior. | Put it behind a narrow plugin/adapter and `cfg`; keep gameplay and shared assets target-agnostic. A successful host build proves only that host. |

## Rendering, UI, text, and audio

| Feature or syntax | Recommendation | When to use it | How to use it correctly |
|---|---|---|---|
| `Camera2d` / `Camera3d` | **Default** | Creating a 2D/3D view. | Spawn the camera component directly with `Camera`, `Projection`, and `Transform` overrides only when needed. |
| `Sprite` | **Default** | Ordinary textured or colored 2D imagery. | Spawn `Sprite` directly; its image handle belongs in the typed `Sprite` component. |
| `Mesh2d` / `Mesh3d` | **Default** | Rendering geometric assets. | Pair with `MeshMaterial2d<M>` or `MeshMaterial3d<M>` and a `Transform`. Reuse handles. |
| Math primitives | **Default** | Procedural standard shapes. | Use current primitives such as `Circle`, `Cuboid`, `Sphere`, and `Cone`; do not use removed `shape::` paths. |
| `StandardMaterial` | **Default** for ordinary 3D surfaces | PBR or unlit standard rendering. | Set only intentional fields. Share materials when equal; split them only when independent mutation is required. |
| Custom `Material` / `AsBindGroup` | **Use when** | A surface model cannot be expressed by `StandardMaterial`. | Keep bindings stable, minimize shader variants, and verify on all selected renderer backends. |
| Material extensions | **Use when** | Extending standard rendering is less work and risk than replacing it. | Reuse the standard pipeline and add the minimum data/shader behavior required. |
| OKLCH color authoring | **Required** | Every project-authored color in code, tests, examples, and documentation. | Use `Color::oklch` for opaque colors and `Color::oklcha` for intentional alpha. Convert only at APIs or shader boundaries that require another representation; perform lighting/blending math in linear space. |
| Visibility components | **Default** | Renderable hierarchies. | Let required components establish visibility; use `Visibility` for intent and respect inherited/view visibility. Do not disable culling casually. |
| `Node` UI | **Default** | Runtime application/game UI. | Spawn `Node` and current UI components directly or author nested UI in BSN. Keep layout data declarative. |
| BSN for UI | **Use when** | UI has nested, repeated, or named structure. | Compose small scene functions/widgets; use `Children [...]`; keep behavioral state in components/systems and observer callbacks focused. |
| Headless widgets | **Default** for reusable interaction semantics | Buttons, sliders, checkboxes, radios, scrollbars, and similar controls. | Separate interaction/accessibility/state semantics from presentation so skins remain replaceable. |
| Feathers | **Use cautiously** | Editor/dev-tool UI that benefits from Bevy's evolving reference skin. | Treat it as evolving rather than the default stable skin for a shipped game UI; isolate styling dependencies. |
| `Text` and editable text | **Use when** | Display or user-editable text is required. | Use current text components/widgets and `EditableText` behavior; keep validation and authoritative domain strings outside presentation when needed. |
| Picking | **Default** for pointer targeting | UI/world selection, hover, drag, or click interaction. | Use Bevy's picking stack and observers; filter pickable entities/layers to reduce ambiguity and cost. |
| `AudioPlayer` | **Default** | Playing an audio source on an entity. | Spawn `AudioPlayer(handle)` with playback settings/components. Enable the `audio` Cargo feature explicitly if defaults are disabled. |
| Gizmos | **Default** for debug drawing | Visualizing bounds, rays, grids, paths, AI, or physics diagnostics. | Keep debug visualization out of authoritative state and compile/configure it appropriately for release. |

## Optimization decision table

Optimization is a budget process: define representative entity counts, message rates, asset sizes, target hardware, and frame-time goals before choosing specialized APIs.

| Area | Recommendation | When it helps | How to apply it | Verification / caveat |
|---|---|---|---|---|
| Frame diagnostics | **Default first step** | Any performance investigation. | Add `FrameTimeDiagnosticsPlugin`, `EntityCountDiagnosticsPlugin`, `SystemInformationDiagnosticsPlugin`, `LogDiagnosticsPlugin`, or `DiagnosticsOverlayPlugin` as appropriate. | Measure representative gameplay, not an empty room. Separate CPU frame time, GPU time, loading stalls, and memory. |
| Development profile | **Default** | Faster debug iteration with usable dependency performance. | Use `[profile.dev] opt-level = 1` and `[profile.dev.package."*"] opt-level = 3`. | Profile release builds before shipping conclusions; debug timings are not production timings. |
| Release profile | **Use when** | Shipping or benchmarking representative builds. | Start with `--release`; consider `codegen-units = 1` and thin LTO after measuring build-time versus runtime/size tradeoffs. | LTO increases build/link cost. Benchmark the same content and target. |
| Dynamic linking | **Use only for development** | Local iteration/link time is a bottleneck. | Enable Bevy's dynamic-linking development path only in local/dev configuration. | Do not ship it as the default deployment strategy. |
| Feature pruning | **Use when** | Compile time, binary size, platform dependency surface, or attack surface matters. | Disable defaults and enable needed high-level collections such as `3d`, `ui`, and `audio`. | Re-test asset formats, windowing, rendering, audio, and every selected target. Missing feature edges can fail at runtime or compile time. |
| Narrow system access | **Default** | Every frame. | Split unrelated resource/component mutation, request immutable data where possible, and use focused queries. | Check scheduler diagnostics/traces if systems serialize unexpectedly. Do not split so far that coordination cost and complexity dominate. |
| Run conditions | **Default for inactive domains** | Menus, paused modes, absent messages, disabled effects, or state-scoped logic. | Use state conditions or specific conditions such as `on_message::<T>`. | A run condition also has cost; use it for meaningful skipped work, not trivial systems without evidence. |
| Change-driven work | **Use when** | Synchronization or expensive recomputation only follows actual changes. | Use `Changed<T>`/`Added<T>`, observers, messages, or explicit dirty markers depending semantics. | Avoid taking `&mut T` and writing identical values; it can create false change signals and downstream work. |
| Bounded queues | **Default** | Input, networking, spawning, telemetry, and gameplay messages. | Cap production/consumption per tick, define overflow/coalescing policy, and monitor backlog. | Unbounded work per frame turns bursts into stalls and harms deterministic replay. |
| Query filtering | **Default** | Large worlds or many archetypes. | Add domain markers and exact `With`/`Without` filters; avoid `Option<&T>` when separate queries express the cases better. | Profile archetype count and iteration; overly fragmented component combinations can also cost. |
| Sparse-set components | **Measure first** | Very frequent add/remove dominates over iteration. | Apply sparse storage to the specific high-churn component. | Table storage is normally faster to iterate. Benchmark both under real churn and query patterns. |
| Parallel iteration | **Measure first** | Large, independent, computationally heavy per-entity work. | Use `par_iter[_mut]().for_each`; make output independent of iteration order. | Small/light loops can get slower from scheduling overhead. Do not use for order-dependent gameplay. |
| Static transform optimization | **Default on** | Many entities remain static. | Leave Bevy's `StaticTransformOptimizations` enabled. | Its tracking can hurt scenes where nearly every transform changes; disable only from a measured regression. |
| Hierarchy depth/churn | **Default: keep intentional** | Transform/UI propagation becomes significant. | Avoid needless wrapper entities, deep chains, and repeated reparenting; model ownership clearly. | Do not flatten useful semantics for hypothetical speed. Measure propagation and layout costs first. |
| Shared meshes/materials | **Default** | Repeated actors, tiles, props, UI images, or effects. | Reuse handles and vary per-instance components only when supported/needed. | Mutating a shared asset changes every user; create a distinct asset only when independent state is intended. |
| Automatic batching/GPU-driven rendering | **Default** | Repeated compatible renderables. | Use compatible shared assets/materials and let Bevy batch/GPU-drive where supported. | Avoid custom instancing until capture/profiling shows the automatic path is insufficient. Backend/hardware support varies. |
| Frustum culling | **Default on** | Ordinary cameras and bounded geometry. | Preserve correct mesh bounds and default culling. | Add `NoFrustumCulling` only for content whose bounds cannot represent visibility, and document the reason. |
| Distance LOD/HLOD | **Use when** | Distant detail or entity count consumes meaningful CPU/GPU time. | Use `VisibilityRange` and deliberately authored lower-detail representations/groups. | Tune thresholds with camera/FOV and popping in mind; test representative motion. |
| Occlusion culling | **Measure first** | Dense 3D scenes have large occluders and substantial hidden geometry. | Add `DepthPrepass` and `OcclusionCulling` to the relevant camera, then tune/test. | Prepass/query overhead can lose on open or simple scenes. In 0.19 it is incompatible with `DeferredPrepass`; combining them has unspecified behavior. Validate supported backends and visual correctness. |
| Shadow participation | **Use when** | Some objects do not need to cast or receive shadows. | Add `NotShadowCaster` or `NotShadowReceiver` only when visually correct. | Shadow savings depend on light count, map resolution, and visible casters. |
| Lights and shadows | **Default: budget explicitly** | Every lit 3D scene. | Bound shadow-casting lights, shadow distances/resolutions, and overlapping dynamic lights. Bake or simplify where the art permits. | Inspect GPU captures and worst-case views; average camera positions hide spikes. |
| Post-processing | **Use when budgeted** | Bloom, SSAO, contact shadows, TAA/FXAA, tonemapping, or custom fullscreen effects add visible value. | Enable per camera and select the cheapest effect/quality meeting the art target. | Stack cost is resolution- and hardware-dependent. Test effect combinations, not just each alone. |
| UI layout churn | **Default: update only real changes** | Large or frequently changing UI. | Keep stable UI entities, change targeted components/text, and avoid rebuilding whole trees every frame. | Measure layout, text shaping, and extraction separately. BSN improves authoring, not automatically runtime cost. |
| Asset loading | **Default asynchronous** | All nontrivial runtime content. | Predeclare/load required handles, expose loading state, stream optional content, and avoid main-thread waits. | Test cold cache, missing/corrupt assets, and slow storage—not only warm developer machines. |
| Asset mutation | **Default: compare before mutate** | Dynamic materials, meshes, images, or other assets. | Mutate only on actual changes; isolate per-instance differences from globally shared assets. | Asset changes can trigger render extraction/upload and break batching. |
| Deterministic fixed simulation | **Default for rule-critical work** | Replay, networking, tests, or stable game feel. | Use bounded intent and fixed ticks; keep render transforms/effects as synchronized views where necessary. | Floating-point and iteration order can still vary; define deterministic ordering and numeric bounds explicitly. |
| Specialized renderer work | **Last resort** | Profiling proves a standard material/pass/batching path cannot meet a hard budget. | Add the smallest render-schedule system/material extension that solves the bottleneck. | Custom render code carries backend, upgrade, and correctness costs. Require a measured budget and regression test/capture. |

## Current syntax examples

### Plugin ownership and explicit schedules

```rust
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MoveIntent>()
            .add_systems(Update, collect_input)
            .add_systems(FixedUpdate, apply_movement);
    }
}

#[derive(Message, Debug, Clone, Copy)]
struct MoveIntent(Vec2);

fn collect_input(mut intents: MessageWriter<MoveIntent>) {
    intents.write(MoveIntent(Vec2::X));
}

fn apply_movement(
    mut intents: MessageReader<MoveIntent>,
    mut movers: Query<&mut Transform, With<Player>>,
) {
    for intent in intents.read() {
        for mut transform in &mut movers {
            transform.translation += intent.0.extend(0.0);
        }
    }
}

#[derive(Component)]
struct Player;
```

In a production fixed-tick path, bound or coalesce the number of intents processed per tick rather than nesting over an unlimited queue.

### Direct current render components

```rust
fn spawn_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Cube"),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::oklch(0.732_032, 0.153_756, 240.894),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.5),
    ));
}
```

### Observer event

```rust
#[derive(Event)]
struct Explode;

fn react_to_explosion(_event: On<Explode>) {
    // Immediate reactive behavior.
}

fn register(app: &mut App) {
    app.add_observer(react_to_explosion);
}

fn trigger(mut commands: Commands) {
    commands.trigger(Explode);
}
```

Use an `EntityEvent` when the event targets a specific entity or propagates through a relationship.

### Current hierarchy

```rust
fn spawn_hierarchy(mut commands: Commands) {
    commands.spawn((
        Name::new("Parent"),
        Transform::default(),
        children![(Name::new("Child"), Transform::from_xyz(1.0, 0.0, 0.0))],
    ));
}
```

Alternatively, insert `ChildOf(parent_entity)` on an existing/spawned child. Read the relationship with `child_of.parent()`; do not mutate `Children` directly.

### Bevy Scene Notation in 0.19

```rust
use bevy::prelude::*;

#[derive(Component, Default, Clone)]
struct Health(u32);

#[derive(Component, Default, Clone)]
struct Sword;

fn player_scene() -> impl Scene {
    bsn! {
        #Player // Adds Name::new("Player") and creates a local entity reference.
        Health(100)
        Children [
            Sword,
        ]
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn_scene(bsn! { player_scene() });
}
```

Use `commands.queue_spawn_scene(...)` instead when the scene declares asset dependencies that may not be loaded. BSN values can use `asset_value(...)`, `template_value(...)`, `FromTemplate`, and string asset paths where the template supports them. There is no first-party `.bsn` file loader in Bevy 0.19.

## Obsolete-to-current migration table

These old forms may appear in tutorials and search results. The replacement column is the only recommended syntax here.

| Avoid / obsolete form | Bevy 0.19 replacement | Reason |
|---|---|---|
| `Camera2dBundle`, `Camera3dBundle`, `PbrBundle`, `SpriteBundle`, `NodeBundle`, `TextBundle`, `SpatialBundle` | Spawn `Camera2d`/`Camera3d`, meshes/materials, `Sprite`, `Node`, current text components, and `Transform` directly. | Required components now supply engine invariants; the old convenience bundle structs are removed/obsolete. |
| A bare `Handle<Mesh>`, `Handle<Scene>`, or other handle as a component | `Mesh3d`, `Mesh2d`, material wrappers, `WorldAssetRoot`, `DynamicWorldRoot`, `AudioPlayer`, or the owning typed component. | Typed components state the asset's entity role and drive current systems. |
| `add_system(...)`, stage APIs, `.system()` | `add_systems(Startup/Update/FixedUpdate, system)` | Explicit typed schedules are the current app model. |
| `EventReader`, `EventWriter`, `add_event`, `send`, `iter` for buffered queues | `MessageReader`, `MessageWriter`, `add_message`, `write`, `read` | Since 0.17, buffered messages and observer events have distinct semantics/APIs. |
| Observer parameter `Trigger<E>` | `On<E>` | `On<E>` is the current observer system parameter. |
| `Replace`, `OnReplace`, `on_replace`, `#[component(on_replace = ...)]` | `Discard`, `on_discard`, `#[component(on_discard = ...)]` | Lifecycle replacement semantics were renamed/currentized. |
| `Query::get_single()` / `get_single_mut()` | `Query::single()` / `single_mut()` returning `Result`, or a filtered `Single` parameter | Current APIs make cardinality handling explicit. |
| `Parent`, `push_children`, direct `Children` mutation | `ChildOf`, `children![...]`, hierarchy builders | `Children` is maintained from the relationship source. |
| `despawn_recursive()` | `despawn()` with current linked child relationships | Current linked-spawn behavior handles descendants. |
| `Input<KeyCode>`, `Input<MouseButton>` | `ButtonInput<KeyCode>`, `ButtonInput<MouseButton>` | Current button-state API. |
| `Gamepads` resource | Query `Gamepad` components | Gamepads are entities. |
| `Color::rgb`, `Color::rgba`, `Color::srgb`, `Color::srgba`, linear literals, named constants, or palettes | `Color::oklch` / `Color::oklcha` | OKLCH is the project's exclusive color-authoring format; convert only at a required renderer/API boundary. |
| `shape::Cube`, `shape::Circle`, and similar paths | `Cuboid`, `Circle`, `Sphere`, `Cone`, and other current math primitives | Current primitive API. |
| `SceneRoot` | `WorldAssetRoot` | In 0.19, static world/scene assets were renamed around world serialization. |
| `DynamicSceneRoot` | `DynamicWorldRoot` | Current dynamic world-asset component name. |
| `DynamicScene` | `DynamicWorld` | Current world-serialization terminology. |
| `SceneBundle` | `WorldAssetRoot` for loaded world assets, or BSN/direct components for code-authored composition | Current scene and required-component model. |
| Camera `RenderGraph` nodes/edges | Render-world systems in `Core2d`/`Core3d` with explicit sets/order | Bevy 0.19 replaced the camera render graph with ordinary render schedules and systems. |
| Assuming `2d`/`3d` enables UI or audio | Enable `ui` and `audio` explicitly | Bevy 0.19 high-level feature collections are independent at these boundaries. |
| Shipping `.bsn` asset files as if core loads them | Use Rust `bsn!`/`bsn_list!` macros | The file format/loader is planned but not released in core Bevy 0.19. |

## Arenic-specific recommendations

The current project already uses BSN in a sensible way:

| Existing area | Recommendation | Why |
|---|---|---|
| `arena() -> impl SceneList` | **Keep** the `bsn_list!` composition. | It makes the initial arena roots explicit and reviewable. |
| `tile::scene`, `hero::scene`, `enemy::boss_scene` | **Keep** these focused scene functions. | They declaratively couple marker components with presentation assets and transforms without moving gameplay rules into scenes. |
| Camera spawning | **Construct cameras with focused BSN scene functions.** Keep runtime activation and reparenting in systems. | BSN keeps static camera configuration reviewable, while systems make resource-driven activation and hierarchy changes explicit. |
| Movement | **Keep systems plus `MoveHero` messages.** | This is device-neutral gameplay intent and simulation, not declarative scene structure. |
| Future HUD, menus, loadouts, actor visual rigs | **Prefer a BSN pilot.** | Nested structure, repeated components, assets, and named entity references are where BSN provides the most leverage. |
| Future external scene authoring | **Wait or isolate an evaluated third-party loader.** | Core 0.19 does not load `.bsn` files. Do not couple production content to a promised future format. |

One repository-policy note should be corrected separately if this guide is adopted: the local `AGENTS.md` says to use `SceneRoot`, but official Bevy 0.19 renamed it to `WorldAssetRoot`. This guide follows the installed 0.19.0 source and the official 0.18-to-0.19 migration guide.

## Primary sources

- [Bevy 0.19 release notes](https://bevy.org/news/bevy-0-19/)
- [Bevy 0.19.0 API documentation](https://docs.rs/bevy/0.19.0/bevy/)
- [Bevy 0.18 to 0.19 migration guide](https://bevy.org/learn/migration-guides/0-18-to-0-19/)
- [Bevy 0.16 to 0.17 messages/events migration](https://bevy.org/learn/migration-guides/0-16-to-0-17/)
- [Official Bevy setup and compile-time configuration](https://bevy.org/learn/quick-start/getting-started/setup/)
- [Official diagnostics example](https://bevy.org/examples/diagnostics/log-diagnostics/)
- [Bevy Scene API and BSN documentation](https://docs.rs/bevy/0.19.0/bevy/scene/)
- [Official Bevy news archive](https://bevy.org/news/)

### Release-note set reviewed

[0.1](https://bevy.org/news/introducing-bevy/) · [0.2](https://bevy.org/news/bevy-0-2/) · [0.3](https://bevy.org/news/bevy-0-3/) · [0.4](https://bevy.org/news/bevy-0-4/) · [0.5](https://bevy.org/news/bevy-0-5/) · [0.6](https://bevy.org/news/bevy-0-6/) · [0.7](https://bevy.org/news/bevy-0-7/) · [0.8](https://bevy.org/news/bevy-0-8/) · [0.9](https://bevy.org/news/bevy-0-9/) · [0.10](https://bevy.org/news/bevy-0-10/) · [0.11](https://bevy.org/news/bevy-0-11/) · [0.12](https://bevy.org/news/bevy-0-12/) · [0.13](https://bevy.org/news/bevy-0-13/) · [0.14](https://bevy.org/news/bevy-0-14/) · [0.15](https://bevy.org/news/bevy-0-15/) · [0.16](https://bevy.org/news/bevy-0-16/) · [0.17](https://bevy.org/news/bevy-0-17/) · [0.18](https://bevy.org/news/bevy-0-18/) · [0.19](https://bevy.org/news/bevy-0-19/)
