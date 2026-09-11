use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

use crate::{theme, tile};

const COLUMNS: u16 = 81;
const ROWS: u16 = 51;
const DOT_RADIUS: f32 = 0.006;
const BACKDROP_Z: f32 = -0.02;

/// Renders the arena grid with one fixed mesh and one shared material.
pub struct ArenaBackdropPlugin;

impl Plugin for ArenaBackdropPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_backdrop);
    }
}

fn spawn_backdrop(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(dot_grid_mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: theme::MUTED,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, BACKDROP_Z),
    ));
}

fn dot_grid_mesh() -> Mesh {
    // The arena has a fixed 20 by 12.5 world-unit footprint. Keep all 4,131
    // markers in one mesh instead of creating an entity and draw per marker.
    let dot_count = usize::from(COLUMNS).strict_mul(usize::from(ROWS));
    let vertex_count = dot_count.strict_mul(4);
    let mut positions = Vec::with_capacity(vertex_count);
    let normals = vec![[0.0, 0.0, 1.0]; vertex_count];
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(dot_count.strict_mul(6));
    let half_width = f32::from(COLUMNS.strict_sub(1)) * tile::SIZE * 0.5;
    let half_height = f32::from(ROWS.strict_sub(1)) * tile::SIZE * 0.5;

    for row in 0..ROWS {
        for column in 0..COLUMNS {
            let x = f32::from(column) * tile::SIZE - half_width;
            let y = f32::from(row) * tile::SIZE - half_height;
            positions.extend([
                [x - DOT_RADIUS, y - DOT_RADIUS, 0.0],
                [x + DOT_RADIUS, y - DOT_RADIUS, 0.0],
                [x + DOT_RADIUS, y + DOT_RADIUS, 0.0],
                [x - DOT_RADIUS, y + DOT_RADIUS, 0.0],
            ]);
            uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

            let first = u32::from(row)
                .strict_mul(u32::from(COLUMNS))
                .strict_add(u32::from(column))
                .strict_mul(4);
            indices.extend([
                first,
                first.strict_add(1),
                first.strict_add(2),
                first,
                first.strict_add(2),
                first.strict_add(3),
            ]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use bevy::mesh::VertexAttributeValues;

    use super::*;

    #[test]
    fn arena_grid_geometry_is_finite_bounded_and_has_valid_indices() {
        let mesh = dot_grid_mesh();
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("invariant: the backdrop mesh contains three-dimensional positions");
        };
        let indices = mesh
            .indices()
            .expect("invariant: the backdrop mesh has triangle indices");

        assert_eq!(positions.len(), 16_524);
        assert_eq!(indices.len(), 24_786);
        assert!(indices.iter().all(|index| index < positions.len()));
        assert!(positions.iter().all(|position| {
            position.iter().all(|coordinate| coordinate.is_finite())
                && position[0].abs() <= 10.006
                && position[1].abs() <= 6.256
                && position[2] == 0.0
        }));
    }
}
