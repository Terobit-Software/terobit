use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use glam::{Quat, Vec3};
use runtime::{
    Runtime, RuntimeContext,
    app::{KeyCode, PhysicalKey},
    ecs::{Camera, Collider, Material, Mesh, RigidBody, Transform},
};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

struct CameraState {
    yaw: f32,
    pitch: f32,
    held: HashSet<KeyCode>,
}

const SENSITIVITY: f32 = 0.002;
const MOVE_SPEED: f32 = 0.15;

fn main() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let mut runtime = Runtime::default();

    let cam = Arc::new(Mutex::new(CameraState {
        yaw: 0.0,
        pitch: 0.0,
        held: HashSet::new(),
    }));

    runtime.on_start(|mut ctx: RuntimeContext| {
        // load before creating scene so the scene borrow doesn't block ctx
        let chair = ctx
            .load_mesh(r"D:\bluffchair.obj")
            .expect("failed to load bluffchair.obj");

        let mut scene = ctx.create_scene("main");

        scene.spawn((Transform::default(), Camera::default()));

        // floor: big flat slab, static rigidbody so falling objects rest on it.
        // uses Mesh::cube() (not plane()) since Collider::cube() is a box centered on
        // the transform, a plane mesh sits flush at y=0 so its top would float 0.5
        // units above the collider's actual surface
        scene.spawn((
            Transform::new()
                .with_pos(Vec3::new(0.0, -50.0, 0.0))
                .with_scale(Vec3::new(400.0, 1.0, 400.0)),
            Mesh::cube(),
            Material::new(0.3, 0.3, 0.35, 1.0),
            RigidBody::new_static(),
            Collider::cube(),
        ));

        for _ in 0..5000 {
            scene.spawn((
                Transform::new()
                    .with_pos(Vec3::new(
                        rand::random_range(-150..150) as f32,
                        rand::random_range(0..250) as f32,
                        rand::random_range(-150..150) as f32,
                    ))
                    .with_scale(Vec3::new(
                        rand::random_range(1..2) as f32,
                        rand::random_range(1..2) as f32,
                        rand::random_range(1..2) as f32,
                    )),
                chair.clone(),
                Material::red(),
                RigidBody::new(2.0).with_restitution(0.0),
                Collider::inherit(),
            ));
        }

        for _ in 0..5000 {
            scene.spawn((
                Transform::new().with_pos(Vec3::new(
                    rand::random_range(-150..150) as f32,
                    rand::random_range(0..200) as f32,
                    rand::random_range(-150..150) as f32,
                )),
                Mesh::cube(),
                Material::blue(),
                RigidBody::new(1.0).with_restitution(0.5),
                Collider::inherit(),
            ));
        }

        ctx.grab_cursor();
    });

    let cam_mouse = cam.clone();
    runtime.on_mouse_motion(move |_ctx, dx, dy| {
        let mut s = cam_mouse.lock().unwrap();
        s.yaw -= dx as f32 * SENSITIVITY;
        // dy positive = mouse moved down = look down = pitch decreases
        s.pitch =
            (s.pitch - dy as f32 * SENSITIVITY).clamp(-89f32.to_radians(), 89f32.to_radians());
    });

    let cam_key = cam.clone();
    runtime.on_key(move |_ctx, key| {
        let mut s = cam_key.lock().unwrap();
        if let PhysicalKey::Code(code) = key.physical_key {
            if key.state.is_pressed() {
                s.held.insert(code);
            } else {
                s.held.remove(&code);
            }
        }
    });

    let cam_frame = cam.clone();
    runtime.on_frame(move |mut ctx: RuntimeContext, _delta: f32| {
        let mut main_scene = ctx.get_scene("main").unwrap();

        // compute camera update from shared state, then drop the lock before touching the world
        let (rotation, delta_pos) = {
            let s = cam_frame.lock().unwrap();

            // full rotation = yaw around world-y * pitch around local-z
            let rotation = Quat::from_rotation_y(s.yaw) * Quat::from_rotation_z(s.pitch);

            // horizontal movement ignores pitch so you walk flat instead of flying
            let forward = Quat::from_rotation_y(s.yaw) * Vec3::X;
            let right = Quat::from_rotation_y(s.yaw) * Vec3::Z;

            let mut delta = Vec3::ZERO;
            if s.held.contains(&KeyCode::KeyW) {
                delta += forward;
            }
            if s.held.contains(&KeyCode::KeyS) {
                delta -= forward;
            }
            if s.held.contains(&KeyCode::KeyD) {
                delta += right;
            }
            if s.held.contains(&KeyCode::KeyA) {
                delta -= right;
            }
            if s.held.contains(&KeyCode::Space) {
                delta.y += 1.0;
            }
            if s.held.contains(&KeyCode::ControlLeft) {
                delta.y -= 1.0;
            }

            // normalize so diagonal movement isn't faster
            if delta.length_squared() > 0.0 {
                delta = delta.normalize();
            }

            let speed = if s.held.contains(&KeyCode::ShiftLeft) {
                MOVE_SPEED * 2.0
            } else {
                MOVE_SPEED
            };
            (rotation, delta * speed)
        };

        for (mut transform, _) in main_scene
            .query::<(&mut Transform, &Camera)>()
            .iter_mut(main_scene.mut_raw())
        {
            transform.position += delta_pos;
            transform.rotation = rotation;
        }
    });

    runtime.start();
}
