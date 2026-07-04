use glam::Vec3;
use rolt::BodyId;

/// physics body for an entity. requires a `Transform` and a `Collider` on the same
/// entity. the actual simulation lives in jolt, this component is just the spawn
/// description (mass, restitution, static or not) plus a cache of the handle jolt
/// assigned it the first time the physics step saw this entity
#[derive(bevy_ecs::component::Component, Clone)]
pub struct RigidBody {
    pub mass: f32,
    pub restitution: f32,
    pub is_static: bool,
    /// filled in by the physics step the first time it registers this entity
    pub(crate) handle: Option<BodyId>,
}

impl RigidBody {
    /// creates a dynamic body with given mass (kg)
    pub fn new(mass: f32) -> Self {
        Self {
            mass,
            restitution: 0.5,
            is_static: false,
            handle: None,
        }
    }

    /// creates a static body (never moves, infinite effective mass)
    pub fn new_static() -> Self {
        Self {
            is_static: true,
            ..Self::new(0.0)
        }
    }

    /// sets restitution (bounciness, 0.0 = no bounce, 1.0 = perfect bounce), builder style
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::new(1.0)
    }
}

/// collision shape for an entity with a `RigidBody`.
///
/// `Box` is an explicit half-extent box (local units, scaled by `Transform.scale`).
/// `Inherit` derives the shape straight from whatever mesh is on this entity's `Mesh`
/// component instead, so any mesh (built-in or asset-loaded) can be used as a physics
/// body. the shape used is that mesh's convex hull, exact concave-vs-concave contacts
/// aren't practical to solve in real time so this is the standard approximation.
#[derive(bevy_ecs::component::Component, Clone)]
pub enum Collider {
    Box { half_extents: Vec3 },
    Inherit,
}

impl Collider {
    /// box with custom half-extents
    pub fn new(half_extents: Vec3) -> Self {
        Self::Box { half_extents }
    }

    /// box matching the built-in unit cube mesh (0.5 half-extent on every axis)
    pub fn cube() -> Self {
        Self::Box { half_extents: Vec3::splat(0.5) }
    }

    /// derives the collision shape (a convex hull) from this entity's `Mesh` component
    pub fn inherit() -> Self {
        Self::Inherit
    }
}

impl Default for Collider {
    fn default() -> Self {
        Self::cube()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_body_is_dynamic_with_given_mass() {
        let b = RigidBody::new(10.0);
        assert_eq!(b.mass, 10.0);
        assert!(!b.is_static);
        assert!(b.handle.is_none());
    }

    #[test]
    fn static_body_is_marked_static() {
        let b = RigidBody::new_static();
        assert!(b.is_static);
    }

    #[test]
    fn with_restitution_sets_field() {
        let b = RigidBody::new(1.0).with_restitution(0.9);
        assert_eq!(b.restitution, 0.9);
    }

    #[test]
    fn cube_collider_half_extent_is_half() {
        let Collider::Box { half_extents } = Collider::cube() else {
            panic!("expected a box collider");
        };
        assert_eq!(half_extents, Vec3::splat(0.5));
    }

    #[test]
    fn default_collider_matches_cube() {
        let (Collider::Box { half_extents: a }, Collider::Box { half_extents: b }) =
            (Collider::default(), Collider::cube())
        else {
            panic!("expected box colliders");
        };
        assert_eq!(a, b);
    }

    #[test]
    fn inherit_collider_is_distinct_from_box() {
        assert!(matches!(Collider::inherit(), Collider::Inherit));
    }
}
