use std::collections::HashMap;
use std::sync::Once;

use bevy_ecs::prelude::Entity;
use bevy_ecs::world::World;
use glam::{Quat, Vec3};
use joltc_sys::*;
use rolt::{
    BroadPhaseLayer, BroadPhaseLayerInterface, ObjectLayer, ObjectLayerPairFilter,
    ObjectVsBroadPhaseLayerFilter, PhysicsSystem,
};

use crate::{
    consts::{MAX_PHYSICS_BODIES, MAX_PHYSICS_DT, PHYSICS_TEMP_ALLOCATOR_BYTES},
    ecs::{Collider, Mesh, RigidBody, Transform},
    profiling::Profiler,
    rendering::MeshSourceId,
};

// no way to set global gravity in this jolt binding yet. jolt's built-in default
// is (0, -9.81, 0) which is what this engine wants anyway, so nothing to configure

/// only two object layers exist: things that never move, and things that do.
/// good enough until the game actually needs more collision filtering
const OL_NON_MOVING: JPC_ObjectLayer = 0;
const OL_MOVING: JPC_ObjectLayer = 1;

const BPL_NON_MOVING: JPC_BroadPhaseLayer = 0;
const BPL_MOVING: JPC_BroadPhaseLayer = 1;
const BPL_COUNT: JPC_BroadPhaseLayer = 2;

struct BroadPhaseLayers;

impl BroadPhaseLayerInterface for BroadPhaseLayers {
    fn get_num_broad_phase_layers(&self) -> u32 {
        BPL_COUNT as u32
    }

    fn get_broad_phase_layer(&self, layer: ObjectLayer) -> BroadPhaseLayer {
        match layer.raw() {
            OL_NON_MOVING => BroadPhaseLayer::new(BPL_NON_MOVING),
            OL_MOVING => BroadPhaseLayer::new(BPL_MOVING),
            _ => unreachable!(),
        }
    }
}

struct ObjectVsBroadPhase;

impl ObjectVsBroadPhaseLayerFilter for ObjectVsBroadPhase {
    fn should_collide(&self, layer1: ObjectLayer, layer2: BroadPhaseLayer) -> bool {
        match layer1.raw() {
            OL_NON_MOVING => layer2.raw() == BPL_MOVING,
            OL_MOVING => true,
            _ => unreachable!(),
        }
    }
}

struct ObjectLayerPair;

impl ObjectLayerPairFilter for ObjectLayerPair {
    fn should_collide(&self, layer1: ObjectLayer, layer2: ObjectLayer) -> bool {
        match layer1.raw() {
            OL_NON_MOVING => layer2.raw() == OL_MOVING,
            OL_MOVING => true,
            _ => unreachable!(),
        }
    }
}

/// exact-bits scale key so identical scales (the common case: most spawned entities
/// share a handful of scales) reuse the same collider shape instead of rebuilding a
/// convex hull per entity
type ScaleKey = (u32, u32, u32);

fn scale_key(scale: Vec3) -> ScaleKey {
    (scale.x.to_bits(), scale.y.to_bits(), scale.z.to_bits())
}

/// jolt bakes mass into the shape's density (there's no separate mass override in
/// this binding), so the cache key has to include mass too. entities that share a
/// mesh, scale AND mass still get a shared shape, just not ones that only share
/// mesh and scale
type ShapeKey = (MeshSourceId, ScaleKey, u32);

fn to_jolt_vec3(v: Vec3) -> JPC_Vec3 {
    JPC_Vec3 {
        x: v.x,
        y: v.y,
        z: v.z,
        _w: v.z,
    }
}

fn to_jolt_rvec3(v: Vec3) -> JPC_RVec3 {
    JPC_RVec3 {
        x: v.x,
        y: v.y,
        z: v.z,
        _w: v.z,
    }
}

fn to_jolt_quat(q: Quat) -> JPC_Quat {
    JPC_Quat {
        x: q.x,
        y: q.y,
        z: q.z,
        w: q.w,
    }
}

fn from_jolt_rvec3(v: JPC_RVec3) -> Vec3 {
    Vec3::new(v.x, v.y, v.z)
}

fn from_jolt_quat(q: JPC_Quat) -> Quat {
    Quat::from_xyzw(q.x, q.y, q.z, q.w)
}

/// axis-aligned bounding box volume, used as a cheap stand-in for a convex hull's
/// actual volume when deriving a density from a target mass. overestimates the
/// real volume (a hull is always inside its own box) so the resulting mass is a
/// little light, not exact, but good enough for gameplay physics
fn aabb_volume(points: &[Vec3]) -> f32 {
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    for p in points {
        min = min.min(*p);
        max = max.max(*p);
    }
    let extents = (max - min).max(Vec3::splat(0.001));
    extents.x * extents.y * extents.z
}

fn create_box_shape(settings: &JPC_BoxShapeSettings) -> Option<*mut JPC_Shape> {
    let mut shape: *mut JPC_Shape = std::ptr::null_mut();
    let mut err: *mut JPC_String = std::ptr::null_mut();
    unsafe {
        if JPC_BoxShapeSettings_Create(settings, &mut shape, &mut err) {
            Some(shape)
        } else {
            None
        }
    }
}

fn create_convex_hull_shape(settings: &JPC_ConvexHullShapeSettings) -> Option<*mut JPC_Shape> {
    let mut shape: *mut JPC_Shape = std::ptr::null_mut();
    let mut err: *mut JPC_String = std::ptr::null_mut();
    unsafe {
        if JPC_ConvexHullShapeSettings_Create(settings, &mut shape, &mut err) {
            Some(shape)
        } else {
            None
        }
    }
}

/// owns the jolt simulation for one scene. long-lived: created once and stepped
/// every frame. entities are registered into it lazily, the first physics step that
/// sees a `RigidBody` without a cached handle inserts it
pub(crate) struct PhysicsScene {
    system: PhysicsSystem,
    temp_allocator: *mut JPC_TempAllocatorImpl,
    job_system: *mut JPC_JobSystemThreadPool,
    /// cache of built collider shapes keyed by (mesh id, scale, mass), so entities
    /// that share those (e.g. a few thousand identical cubes) don't each pay for
    /// their own convex-hull computation
    shape_cache: HashMap<ShapeKey, *mut JPC_Shape>,
}

impl PhysicsScene {
    pub fn new() -> Self {
        static JOLT_INIT: Once = Once::new();
        JOLT_INIT.call_once(|| {
            rolt::register_default_allocator();
            rolt::factory_init();
            rolt::register_types();
        });

        let mut system = PhysicsSystem::new();

        system.init(
            MAX_PHYSICS_BODIES,
            0,
            MAX_PHYSICS_BODIES,
            MAX_PHYSICS_BODIES,
            BroadPhaseLayers,
            ObjectVsBroadPhase,
            ObjectLayerPair,
        );

        let temp_allocator = unsafe { JPC_TempAllocatorImpl_new(PHYSICS_TEMP_ALLOCATOR_BYTES) };
        let job_system = unsafe {
            JPC_JobSystemThreadPool_new2(JPC_MAX_PHYSICS_JOBS as _, JPC_MAX_PHYSICS_BARRIERS as _)
        };

        Self {
            system,
            temp_allocator,
            job_system,
            shape_cache: HashMap::new(),
        }
    }

    /// registers any not-yet-registered rigid bodies, steps the simulation, then
    /// writes the resulting transforms back onto the ecs. dt is clamped so a stalled
    /// frame (window drag, resize) doesn't blow up the integration
    pub fn step(
        &mut self,
        scene: &mut World,
        dt: f32,
        mesh_points: &HashMap<MeshSourceId, Vec<Vec3>>,
        profiler: &mut Profiler,
    ) {
        let dt = dt.min(MAX_PHYSICS_DT);
        if dt <= 0.0 {
            return;
        }

        profiler.start("physics/register");
        self.register_new_bodies(scene, mesh_points);
        profiler.stop("physics/register");

        profiler.start("physics/update");
        unsafe {
            self.system
                .update(dt, 1, self.temp_allocator, self.job_system);
        }
        profiler.stop("physics/update");

        profiler.start("physics/sync");
        self.sync_transforms(scene);
        profiler.stop("physics/sync");
    }

    fn register_new_bodies(
        &mut self,
        scene: &mut World,
        mesh_points: &HashMap<MeshSourceId, Vec<Vec3>>,
    ) {
        let mut query = scene.query::<(Entity, &Transform, &RigidBody, &Collider, Option<&Mesh>)>();

        let pending: Vec<_> = query
            .iter(scene)
            .filter(|(_, _, body, ..)| body.handle.is_none())
            .map(|(entity, transform, body, collider, mesh)| {
                (
                    entity,
                    *transform,
                    body.mass,
                    body.restitution,
                    body.is_static,
                    collider.clone(),
                    mesh.map(|m| *m.get_id()),
                )
            })
            .collect();

        for (entity, transform, mass, restitution, is_static, collider, mesh_id) in pending {
            let Some(shape) =
                self.shape_for(&collider, mesh_id, transform.scale, mass, mesh_points)
            else {
                continue;
            };

            let body_interface = self.system.body_interface();

            let settings = JPC_BodyCreationSettings {
                Position: to_jolt_rvec3(transform.position),
                Rotation: to_jolt_quat(transform.rotation),
                ObjectLayer: if is_static { OL_NON_MOVING } else { OL_MOVING },
                MotionType: if is_static {
                    JPC_MOTION_TYPE_STATIC
                } else {
                    JPC_MOTION_TYPE_DYNAMIC
                },
                Friction: 0.6,
                Restitution: restitution,
                Shape: shape,
                ..Default::default()
            };

            // safety: `settings.Shape` was just built above and is a valid, live shape pointer
            let body = unsafe { body_interface.create_body(&settings) };
            if body.as_raw().is_null() {
                continue;
            }
            let handle = body.id();

            let activation = if is_static {
                JPC_ACTIVATION_DONT_ACTIVATE
            } else {
                JPC_ACTIVATION_ACTIVATE
            };
            body_interface.add_body(handle, activation);

            if let Some(mut rb) = scene.get_mut::<RigidBody>(entity) {
                rb.handle = Some(handle);
            }
        }
    }

    /// builds (or reuses a cached) collider shape for the given collider descriptor.
    /// `mass` drives the shape's density since jolt has no separate mass override
    fn shape_for(
        &mut self,
        collider: &Collider,
        mesh_id: Option<MeshSourceId>,
        scale: Vec3,
        mass: f32,
        mesh_points: &HashMap<MeshSourceId, Vec<Vec3>>,
    ) -> Option<*mut JPC_Shape> {
        match collider {
            Collider::Box { half_extents } => {
                let e = (*half_extents * scale.abs()).max(Vec3::splat(0.001));
                let volume = (8.0 * e.x * e.y * e.z).max(f32::EPSILON);

                let settings = JPC_BoxShapeSettings {
                    Density: mass.max(0.001) / volume,
                    HalfExtent: to_jolt_vec3(e),
                    ..Default::default()
                };
                create_box_shape(&settings)
            }
            Collider::Inherit => {
                let mesh_id = mesh_id?;
                let key = (mesh_id, scale_key(scale), mass.to_bits());

                if let Some(cached) = self.shape_cache.get(&key) {
                    return Some(*cached);
                }

                let points = mesh_points.get(&mesh_id)?;
                let scaled: Vec<Vec3> = points.iter().map(|p| *p * scale).collect();
                let volume = aabb_volume(&scaled).max(f32::EPSILON);
                let jolt_points: Vec<JPC_Vec3> = scaled.iter().map(|p| to_jolt_vec3(*p)).collect();

                let settings = JPC_ConvexHullShapeSettings {
                    Density: mass.max(0.001) / volume,
                    Points: jolt_points.as_ptr(),
                    PointsLen: jolt_points.len(),
                    ..Default::default()
                };
                // safety: `jolt_points` outlives this call, `Create` copies what it needs
                let shape = create_convex_hull_shape(&settings)?;

                self.shape_cache.insert(key, shape);
                Some(shape)
            }
        }
    }

    fn sync_transforms(&self, scene: &mut World) {
        let body_interface = self.system.body_interface();
        let raw = body_interface.as_raw();

        let mut query = scene.query::<(&mut Transform, &RigidBody)>();
        for (mut transform, body) in query.iter_mut(scene) {
            if body.is_static {
                continue;
            }
            let Some(handle) = body.handle else { continue };

            // safety: `raw` is a live body interface for the whole scope of `self`, `handle`
            // was assigned by this same physics system in `register_new_bodies`
            unsafe {
                transform.position =
                    from_jolt_rvec3(JPC_BodyInterface_GetPosition(raw, handle.raw()));
                transform.rotation =
                    from_jolt_quat(JPC_BodyInterface_GetRotation(raw, handle.raw()));
            }
        }
    }
}

impl Drop for PhysicsScene {
    fn drop(&mut self) {
        unsafe {
            JPC_JobSystemThreadPool_delete(self.job_system);
            JPC_TempAllocatorImpl_delete(self.temp_allocator);
        }
    }
}
