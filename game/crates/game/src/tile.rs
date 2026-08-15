use bevy::prelude::*;

/// Distance between tile centers in the Arenic grid.
pub const SIZE: f32 = 0.25;

const DOT_RADIUS: f32 = SIZE * 0.048;
const Z: f32 = 0.02;

#[derive(Component, Default, Clone, Copy)]
pub struct Tile;

/// Describes the arena's origin tile marker.
pub fn scene() -> impl Scene {
    let material = StandardMaterial {
        base_color: Color::oklch(1.0, 0.0, 0.0),
        unlit: true,
        ..default()
    };

    bsn! {
        Tile
        Mesh3d(asset_value(Circle::new(DOT_RADIUS)))
        MeshMaterial3d::<StandardMaterial>(asset_value(material))
        Transform::from_xyz(0.0, 0.0, Z)
    }
}
