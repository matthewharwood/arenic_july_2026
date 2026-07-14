use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::tile;

/// Radius of the boss's one-tile-wide cone base.
const RADIUS: f32 = tile::SIZE * 0.5;
const HEIGHT: f32 = tile::SIZE;
const TILE_OFFSET_FROM_HERO: IVec2 = IVec2::new(3, 2);

/// Marks a character as hostile to the player.
#[derive(Component, Debug, Clone, Copy)]
pub struct Enemy;

/// Marks an enemy as a boss encounter.
#[derive(Component, Debug, Clone, Copy)]
pub struct Boss;

pub fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    let mesh = meshes.add(Cone::new(RADIUS, HEIGHT));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.15, 0.2),
        unlit: true,
        ..default()
    });
    let position = TILE_OFFSET_FROM_HERO.as_vec2() * tile::SIZE;

    commands
        .spawn((
            Enemy,
            Boss,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(position.x, position.y, HEIGHT * 0.5)
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
        ))
        .id()
}
