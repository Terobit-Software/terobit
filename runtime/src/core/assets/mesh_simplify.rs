use std::collections::HashMap;

use meshopt::{SimplifyOptions, VertexDataAdapter};

use crate::rendering::VertexRaw;

/// collapses the mesh down to roughly `ratio` of its original triangle count via
/// meshoptimizer, then compacts the vertex buffer so only vertices the surviving
/// triangles still reference get uploaded. `target_error` is the max allowed
/// deviation as a fraction of the mesh extents
pub(crate) fn simplify_mesh(
    vertices: &[VertexRaw],
    indices: &[u32],
    ratio: f32,
    target_error: f32,
) -> anyhow::Result<(Vec<VertexRaw>, Vec<u32>)> {
    let ratio = ratio.clamp(0.0, 1.0);
    // rounded down to whole triangles
    let target_index_count = ((indices.len() as f32 * ratio) as usize / 3) * 3;

    let adapter = VertexDataAdapter::new(
        bytemuck::cast_slice(vertices),
        std::mem::size_of::<VertexRaw>(),
        0,
    )
    .map_err(|e| anyhow::anyhow!("meshopt rejected vertex data: {e}"))?;

    // prune lets the simplifier drop tiny disconnected shells entirely instead of
    // preserving each one's border edges
    let mut simplified = meshopt::simplify(
        indices,
        &adapter,
        target_index_count,
        target_error,
        SimplifyOptions::Prune,
        None,
    );

    // scan-style meshes are often a soup of disconnected shells the topology-preserving
    // simplifier refuses to collapse across, stalling way above target. the sloppy
    // variant ignores topology and actually reaches the requested count
    if simplified.len() > target_index_count + target_index_count / 5 {
        simplified = meshopt::simplify_sloppy(
            indices,
            &adapter,
            target_index_count,
            target_error,
            None,
        );
    }

    // remap old indices to a fresh dense vertex buffer
    let mut remap: HashMap<u32, u32> = HashMap::new();
    let mut new_vertices: Vec<VertexRaw> = Vec::new();
    let new_indices: Vec<u32> = simplified
        .iter()
        .map(|&old| {
            *remap.entry(old).or_insert_with(|| {
                new_vertices.push(vertices[old as usize]);
                (new_vertices.len() - 1) as u32
            })
        })
        .collect();

    Ok((new_vertices, new_indices))
}

#[cfg(test)]
mod tests {
    use super::*;

    // dense grid of quads on the xz plane, plenty of redundant coplanar
    // triangles for the simplifier to collapse
    fn grid_mesh(n: usize) -> (Vec<VertexRaw>, Vec<u32>) {
        let vertices = (0..n * n)
            .map(|i| VertexRaw {
                position: [(i % n) as f32, 0.0, (i / n) as f32, 1.0],
                tex_coords: [0.0, 0.0],
                normals: [0.0, 1.0, 0.0],
            })
            .collect();

        let indices = (0..(n - 1) * (n - 1))
            .flat_map(|cell| {
                let x = (cell % (n - 1)) as u32;
                let z = (cell / (n - 1)) as u32;
                let a = z * n as u32 + x;
                let b = a + 1;
                let c = a + n as u32;
                let d = c + 1;
                [a, c, b, b, c, d]
            })
            .collect();

        (vertices, indices)
    }

    #[test]
    fn reduces_triangle_count() {
        let (vertices, indices) = grid_mesh(16);
        let (_, new_indices) = simplify_mesh(&vertices, &indices, 0.25, 0.05).unwrap();
        assert!(new_indices.len() < indices.len());
        assert!(!new_indices.is_empty());
    }

    #[test]
    fn output_is_whole_triangles_with_valid_indices() {
        let (vertices, indices) = grid_mesh(8);
        let (new_vertices, new_indices) = simplify_mesh(&vertices, &indices, 0.5, 0.05).unwrap();
        assert_eq!(new_indices.len() % 3, 0);
        assert!(
            new_indices
                .iter()
                .all(|&i| (i as usize) < new_vertices.len())
        );
    }

    #[test]
    fn compaction_drops_unreferenced_vertices() {
        let (vertices, indices) = grid_mesh(16);
        let (new_vertices, new_indices) = simplify_mesh(&vertices, &indices, 0.1, 0.05).unwrap();
        // every surviving vertex is referenced at least once
        let referenced: std::collections::HashSet<u32> = new_indices.iter().copied().collect();
        assert_eq!(referenced.len(), new_vertices.len());
    }

    #[test]
    fn ratio_one_keeps_geometry_intact() {
        let (vertices, indices) = grid_mesh(4);
        let (_, new_indices) = simplify_mesh(&vertices, &indices, 1.0, 0.0).unwrap();
        assert_eq!(new_indices.len(), indices.len());
    }
}
