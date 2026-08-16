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
    /// Derives the nearest valid grid cell from a finite world-space position.
    pub fn from_world_xy(world_xy: Vec2) -> Option<Self> {
        let grid_xy = world_xy / SIZE;
        if !grid_xy.is_finite() {
            return None;
        }

        Some(Self(IVec2::new(
            rounded_grid_coordinate(grid_xy.x)?,
            rounded_grid_coordinate(grid_xy.y)?,
        )))
    }

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

/// Marks an entity that must receive authoritative grid placement before
/// participating in fixed-tick gameplay.
#[derive(Component, Debug, Default, Clone, Copy)]
#[require(Transform)]
pub struct GridActor;

fn rounded_grid_coordinate(coordinate: f32) -> Option<i32> {
    let rounded = f64::from(coordinate).round();
    (f64::from(i32::MIN)..=f64::from(i32::MAX))
        .contains(&rounded)
        .then_some(rounded as i32)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_position_derives_the_nearest_signed_grid_cell() {
        let world_xy = Vec2::new(SIZE * 2.1, SIZE * -3.2);

        assert_eq!(
            GridPosition::from_world_xy(world_xy),
            Some(GridPosition(IVec2::new(2, -3)))
        );
    }

    #[test]
    fn non_finite_world_position_has_no_grid_cell() {
        assert_eq!(GridPosition::from_world_xy(Vec2::NAN), None);
    }
}
