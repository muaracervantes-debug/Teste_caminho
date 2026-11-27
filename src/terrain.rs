use glam::Vec3;

use crate::renderer::Vertex;

const GRID_CELLS: u32 = 64;
const TERRAIN_SIZE: f32 = 32.0;
const HALF_SIZE: f32 = TERRAIN_SIZE * 0.5;

pub struct TerrainMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// Gera uma malha de terreno de demonstração. A função de altura é provisória:
/// será substituída pelo gerador procedural com seed na etapa correspondente.
pub fn build_mesh() -> TerrainMesh {
    let side = GRID_CELLS + 1;
    let spacing = TERRAIN_SIZE / GRID_CELLS as f32;
    let mut vertices = Vec::with_capacity((side * side) as usize);

    for z in 0..=GRID_CELLS {
        for x in 0..=GRID_CELLS {
            let world_x = -HALF_SIZE + x as f32 * spacing;
            let world_z = -HALF_SIZE + z as f32 * spacing;
            let height = sample_height(world_x, world_z);

            let left = sample_height(world_x - spacing, world_z);
            let right = sample_height(world_x + spacing, world_z);
            let back = sample_height(world_x, world_z - spacing);
            let front = sample_height(world_x, world_z + spacing);

            let normal = Vec3::new(left - right, 2.0 * spacing, back - front)
                .normalize_or_zero();

            vertices.push(Vertex {
                position: [world_x, height, world_z],
                color: terrain_color(height),
                normal: normal.to_array(),
            });
        }
    }

    let mut indices = Vec::with_capacity((GRID_CELLS * GRID_CELLS * 6) as usize);
    for z in 0..GRID_CELLS {
        for x in 0..GRID_CELLS {
            let top_left = z * side + x;
            let top_right = top_left + 1;
            let bottom_left = top_left + side;
            let bottom_right = bottom_left + 1;

            // Ordem anti-horaria vista de cima: normais apontam para +Y.
            indices.extend_from_slice(&[
                top_left,
                bottom_left,
                top_right,
                top_right,
                bottom_left,
                bottom_right,
            ]);
        }
    }

    TerrainMesh { vertices, indices }
}

fn sample_height(x: f32, z: f32) -> f32 {
    (x * 0.22).sin() * (z * 0.18).cos() * 1.15
        + ((x + z) * 0.11).sin() * 0.65
        + ((x - z) * 0.15).cos() * 0.35
}

fn terrain_color(height: f32) -> [f32; 3] {
    if height < -0.55 {
        [0.22, 0.31, 0.22]
    } else if height < 0.55 {
        [0.28, 0.48, 0.27]
    } else if height < 1.25 {
        [0.48, 0.49, 0.29]
    } else {
        [0.48, 0.45, 0.40]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_has_expected_vertex_and_index_counts() {
        let mesh = build_mesh();
        let expected_side = (GRID_CELLS + 1) as usize;

        assert_eq!(mesh.vertices.len(), expected_side * expected_side);
        assert_eq!(mesh.indices.len(), (GRID_CELLS * GRID_CELLS * 6) as usize);
    }

    #[test]
    fn generated_vertices_and_normals_are_finite() {
        let mesh = build_mesh();

        for vertex in mesh.vertices {
            assert!(vertex.position.iter().all(|value| value.is_finite()));
            assert!(vertex.normal.iter().all(|value| value.is_finite()));

            let normal = Vec3::from_array(vertex.normal);
            assert!((normal.length() - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn indices_stay_inside_vertex_buffer() {
        let mesh = build_mesh();
        let vertex_count = mesh.vertices.len() as u32;

        assert!(mesh.indices.iter().all(|index| *index < vertex_count));
    }
}
