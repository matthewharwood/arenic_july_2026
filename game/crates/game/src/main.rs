mod arena_backdrop;
mod camera;
mod enemy;
mod hero;
mod hud;
mod movement;
mod roster;
mod theme;
mod tile;

use bevy::prelude::*;
use bevy::window::WindowResolution;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Arenic".into(),
                resolution: WindowResolution::new(1280, 720),
                #[cfg(target_arch = "wasm32")]
                canvas: Some("#game-canvas".into()),
                #[cfg(target_arch = "wasm32")]
                fit_canvas_to_parent: true,
                #[cfg(target_arch = "wasm32")]
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            camera::GameCameraPlugin,
            movement::HeroMovementPlugin,
            arena_backdrop::ArenaBackdropPlugin,
            hud::HudPlugin,
        ))
        .add_systems(Startup, arena.spawn())
        .run();
}

/// Describes the entities that make up the initial arena.
fn arena() -> impl SceneList {
    bsn_list![tile::scene(), hero::scene(), enemy::boss_scene()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::scene::ScenePlugin;

    #[test]
    fn arena_scene_spawns_each_rendered_gameplay_entity() {
        let mut app = App::new();
        app.add_plugins((
            TaskPoolPlugin::default(),
            AssetPlugin::default(),
            ScenePlugin,
        ))
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .add_systems(Startup, arena.spawn());

        app.update();

        assert_single_rendered_entity::<tile::Tile>(&mut app);
        assert_single_rendered_entity::<hero::Hero>(&mut app);
        assert_single_rendered_entity::<enemy::Boss>(&mut app);
    }

    fn assert_single_rendered_entity<M: Component>(app: &mut App) {
        let world = app.world_mut();
        let mut query = world
            .query_filtered::<(&Mesh3d, &MeshMaterial3d<StandardMaterial>, &Transform), With<M>>();

        query
            .single(world)
            .expect("invariant: the arena scene has one rendered entity for this marker");
    }
}
