use bevy::prelude::*;

/// Distance between tile centers in the Arenic grid.
pub const SIZE: f32 = 0.25;

const DOT_RADIUS: f32 = SIZE * 0.048;
const Z: f32 = 0.02;

#[derive(Component)]
pub struct Tile;

pub fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    let mesh = meshes.add(Circle::new(DOT_RADIUS));
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });

    commands
        .spawn((
            Tile,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, 0.0, Z),
        ))
        .id()
}
