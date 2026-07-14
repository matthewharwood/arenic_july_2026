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
        .add_plugins((camera::GameCameraPlugin, movement::HeroMovementPlugin))
        .add_systems(Startup, (spawn_tile, spawn_hero, spawn_enemy_boss))
        .run();
}

fn spawn_tile(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    tile::spawn(&mut commands, &mut meshes, &mut materials);
}

fn spawn_hero(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    hero::spawn(&mut commands, &mut meshes, &mut materials);
}

fn spawn_enemy_boss(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    enemy::spawn(&mut commands, &mut meshes, &mut materials);
}
