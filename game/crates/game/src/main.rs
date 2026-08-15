mod abilities;
mod camera;
mod enemy;
mod hero;
mod movement;
mod tile;

use bevy::prelude::*;
use bevy::window::WindowResolution;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Arenic".into(),
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            camera::GameCameraPlugin,
            movement::HeroMovementPlugin,
            abilities::HeroAbilitiesPlugin,
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
        assert_initial_hero_abilities(&mut app);
    }

    fn assert_single_rendered_entity<M: Component>(app: &mut App) {
        let world = app.world_mut();
        let mut query = world
            .query_filtered::<(&Mesh3d, &MeshMaterial3d<StandardMaterial>, &Transform), With<M>>();

        query
            .single(world)
            .expect("invariant: the arena scene has one rendered entity for this marker");
    }

    fn assert_initial_hero_abilities(app: &mut App) {
        let world = app.world_mut();
        let mut query = world.query_filtered::<&abilities::Abilities, With<hero::Hero>>();
        let hero_abilities = query
            .single(world)
            .expect("invariant: the arena scene has exactly one hero with abilities");

        assert_eq!(
            hero_abilities.slots,
            [Some(abilities::Ability::Bash), None, None, None,]
        );
    }
}
