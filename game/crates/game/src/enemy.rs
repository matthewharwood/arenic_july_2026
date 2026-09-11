use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::tile::{self, GridActor, GridPosition};

/// Radius of the boss's one-tile-wide cone base.
const RADIUS: f32 = tile::SIZE * 0.5;
const HEIGHT: f32 = tile::SIZE;
const TILE_OFFSET_FROM_HERO: IVec2 = IVec2::new(3, 2);

/// Marks a character as hostile to the player.
#[derive(Component, Debug, Default, Clone, Copy)]
#[require(GridActor)]
pub struct Enemy;

/// Marks an enemy as a boss encounter.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Boss;

/// Describes the boss encounter entity in the initial arena.
pub fn boss_scene() -> impl Scene {
    let material = StandardMaterial {
        base_color: Color::oklch(0.595_752, 0.222_113, 24.719),
        unlit: true,
        ..default()
    };
    let position = TILE_OFFSET_FROM_HERO.as_vec2() * tile::SIZE;

    bsn! {
        Enemy
        Boss
        GridPosition(TILE_OFFSET_FROM_HERO)
        Mesh3d(asset_value(Cone::new(RADIUS, HEIGHT)))
        MeshMaterial3d::<StandardMaterial>(asset_value(material))
        Transform {
            translation: Vec3::new(position.x, position.y, HEIGHT * 0.5),
            rotation: Quat::from_rotation_x(FRAC_PI_2),
        }
    }
}
