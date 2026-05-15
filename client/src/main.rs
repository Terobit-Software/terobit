use glam::{Quat, Vec3};
use runtime::{
    Runtime, RuntimeContext,
    ecs::{Camera, Mesh, Transform},
};
use chrono::Utc;
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // let _profiler = dhat::Profiler::new_heap();

    let mut runtime = Runtime::default();

    runtime.on_start(|mut ctx: RuntimeContext| {
        let mut scene = ctx.create_scene("main");

        scene.spawn((
            Transform::default(),
            Camera::default()
        ));

        scene.spawn((
            Transform::new().with_pos(Vec3 { x: 0.0, y: 0.0, z: 0.0 }),
            Mesh::cube()
        ));

        scene.spawn((
            Transform::new().with_pos(Vec3::new(1.1, 0.0, 0.0)).with_scale(Vec3::new(1.0, 1.0, 1.0)),
            Mesh::cube()
        ));
    });

    runtime.on_frame(|mut ctx: RuntimeContext, delta: f32| {
        let mut main_scene = ctx.get_scene("main").unwrap();

        for (mut transform, camera) in main_scene.query::<(&mut Transform, &Camera)>().iter_mut(main_scene.mut_raw()) {
            let mut pos = Vec3::default();

            pos.y = 2. * (Utc::now().timestamp_millis() as f64 / 1000.).sin() as f32;
            pos.x = 5. * (Utc::now().timestamp_millis() as f64 / 1000.).sin() as f32;
            pos.z = 5. * (Utc::now().timestamp_millis() as f64 / 1000.).cos() as f32;

            transform.position = pos;
        }
    });

    runtime.start();
}
