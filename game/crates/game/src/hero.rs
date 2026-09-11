use bevy::prelude::*;

use crate::tile;

/// Radius of a one-tile-wide hero.
pub const RADIUS: f32 = tile::SIZE * 0.5;

/// Marks the player-controlled character.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Hero;

/// Marks a hero as part of the player's current selection.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Selected;

/// Marks a selected hero as currently able to receive player actions.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Active;

/// Describes the initially selected, player-controlled hero.
pub fn scene() -> impl Scene {
    let material = StandardMaterial {
        base_color: crate::theme::SELECTED,
        unlit: true,
        ..default()
    };

    bsn! {
        Hero
        Selected
        Active
        Mesh3d(asset_value(Sphere::new(RADIUS)))
        MeshMaterial3d::<StandardMaterial>(asset_value(material))
        Transform::from_xyz(0.0, 0.0, RADIUS)
    }
}
