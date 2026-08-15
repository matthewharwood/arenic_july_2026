use bevy::prelude::*;

/// Distance between tile centers in the Arenic grid.
pub const SIZE: f32 = 0.25;

const DOT_RADIUS: f32 = SIZE * 0.048;
const Z: f32 = 0.02;

/// A signed tile coordinate on the arena's Cartesian grid.
///
/// Gameplay rules use this integer coordinate as authoritative state. Rendered
/// transforms may temporarily move away from it for presentation effects.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition(pub IVec2);

impl GridPosition {
    pub fn checked_offset(self, offset: IVec2) -> Option<Self> {
        Some(Self(IVec2::new(
            self.0.x.checked_add(offset.x)?,
            self.0.y.checked_add(offset.y)?,
        )))
    }

    pub fn world_xy(self) -> Vec2 {
        self.0.as_vec2() * SIZE
    }
}

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
