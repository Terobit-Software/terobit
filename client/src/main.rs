use bevy_ecs::entity::Entity;
use glam::{Quat, Vec3};
use rand::rng;
use runtime::{
    Runtime, RuntimeContext, app::{KeyCode, KeyInput, PhysicalKey}, ecs::{Camera, Mesh, Transform}
};
use chrono::Utc;
use rayon::prelude::*;
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

        for i in 0..50000 {
            scene.spawn((
                Transform::new().with_pos(Vec3 { x: rand::random_range(-150..150) as f32, y: rand::random_range(-150..150) as f32, z: rand::random_range(-150..150) as f32 }),
                Mesh::cube()
            ));
        }
    });

    runtime.on_frame(|mut ctx: RuntimeContext, delta: f32| {
        // let mut main_scene = ctx.get_scene("main").unwrap();

        // // for (mut transform, camera) in main_scene.query::<(&mut Transform, &Camera)>().iter_mut(main_scene.mut_raw()) {
        // //     let mut pos = Vec3::default();

        // //     pos.y = 15. * (Utc::now().timestamp_millis() as f64 / 1000.).sin() as f32;
        // //     pos.x = 50. * (Utc::now().timestamp_millis() as f64 / 1000.).sin() as f32;
        // //     pos.z = 50. * (Utc::now().timestamp_millis() as f64 / 1000.).cos() as f32;

        // //     transform.position = pos;
        // // }

        // let t0 = std::time::Instant::now();

        // // main_scene.query::<(&mut Transform, &Mesh)>().par_iter_mut(main_scene.mut_raw()).for_each(
        // //     |(mut transform, mesh)| {
        // //         transform.position.y -= 0.1;
        // //     }
        // // );

        // println!("transforms: {:?}", t0.elapsed());
        // println!("sz: {}", std::mem::size_of::<Entity>())
    });

    runtime.on_key(|mut ctx: RuntimeContext, key_code| {
        let mut main_scene = ctx.get_scene("main").unwrap();
        match key_code.physical_key {
            PhysicalKey::Code(code) => {
                match code {
                    KeyCode::KeyA => {
                        for (mut transform, camera) in main_scene.query::<(&mut Transform, &Camera)>().iter_mut(main_scene.mut_raw()) {
                            transform.position.z -= 0.5;
                        }
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    });

    runtime.start();
}
