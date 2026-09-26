use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};

// planes are left, right, bottom, top, near, far
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct Frustum {
    planes: [FrustumPlane; 6],
}

// xyz is normal, w is distance
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct FrustumPlane {
    normal: [f32; 3],
    distance: f32,
}

impl FrustumPlane {
    fn from_coefficients(coefficients: Vec4) -> Self {
        let length = coefficients.truncate().length();
        let normalized = if length > 0.0 {
            coefficients / length
        } else {
            coefficients
        };

        Self {
            normal: normalized.truncate().to_array(),
            distance: normalized.w,
        }
    }

    pub(crate) fn normal(&self) -> Vec3 {
        Vec3::from_array(self.normal)
    }

    pub(crate) fn distance(&self) -> f32 {
        self.distance
    }

    // negative means outside
    pub(crate) fn signed_distance(&self, point: Vec3) -> f32 {
        self.normal().dot(point) + self.distance
    }
}

impl Frustum {
    pub(crate) fn from_view_proj(view_proj: Mat4) -> Self {
        let r0 = view_proj.row(0);
        let r1 = view_proj.row(1);
        let r2 = view_proj.row(2);
        let r3 = view_proj.row(3);

        Self {
            planes: [
                FrustumPlane::from_coefficients(r3 + r0), // left
                FrustumPlane::from_coefficients(r3 - r0), // right
                FrustumPlane::from_coefficients(r3 + r1), // bottom
                FrustumPlane::from_coefficients(r3 - r1), // top
                FrustumPlane::from_coefficients(r2),      // near ([0, 1] depth)
                FrustumPlane::from_coefficients(r3 - r2), // far
            ],
        }
    }

    pub(crate) fn planes(&self) -> &[FrustumPlane; 6] {
        &self.planes
    }

    pub(crate) fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.signed_distance(center) >= -radius)
    }

    pub(crate) fn contains_point(&self, point: Vec3) -> bool {
        self.intersects_sphere(point, 0.0)
    }
}
