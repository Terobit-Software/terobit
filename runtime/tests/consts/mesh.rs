#[cfg(test)]
mod tests {
    use runtime::consts::mesh::*;

    #[test]
    fn cube_has_six_faces() {
        assert_eq!(CUBE_FACES.len(), 6);
    }

    #[test]
    fn every_face_has_four_corners() {
        for (_, corners) in CUBE_FACES {
            assert_eq!(corners.len(), 4);
        }
    }

    #[test]
    fn every_face_normal_is_axis_aligned_and_unit_length() {
        for (normal, _) in CUBE_FACES {
            let axis_aligned_components =
                normal.iter().filter(|component| **component == 0.0).count();
            assert_eq!(axis_aligned_components, 2);

            let length_squared: f32 = normal.iter().map(|component| component * component).sum();
            assert_eq!(length_squared, 1.0);
        }
    }

    #[test]
    fn face_normals_cover_all_six_directions() {
        let mut normals: Vec<[f32; 3]> = CUBE_FACES.iter().map(|(normal, _)| *normal).collect();
        normals.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut expected = [
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ];
        expected.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(normals, expected);
    }
}
