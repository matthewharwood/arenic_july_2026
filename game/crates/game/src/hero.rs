use bevy::prelude::*;

use crate::tile;

/// Radius of a one-tile-wide hero.
pub const RADIUS: f32 = tile::SIZE * 0.5;

/// Marks the player-controlled character.
#[derive(Component, Debug, Clone, Copy)]
pub struct Hero;

/// Marks a hero as part of the player's current selection.
#[derive(Component, Debug, Clone, Copy)]
pub struct Selected;

/// Marks a selected hero as currently able to receive player actions.
#[derive(Component, Debug, Clone, Copy)]
pub struct Active;

pub fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    let mesh = meshes.add(Sphere::new(RADIUS));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.7, 1.0),
        unlit: true,
        ..default()
    });

    commands
        .spawn((
            Hero,
            Selected,
            Active,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, 0.0, RADIUS),
        ))
        .id()
}
