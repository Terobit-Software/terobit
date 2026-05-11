use glam::Vec3;
use runtime::{Runtime, RuntimeContext, ecs::Transform};
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;


fn main() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // let _profiler = dhat::Profiler::new_heap();

    let mut runtime = Runtime::default();

    runtime.on_start(
        |mut ctx: RuntimeContext| {
            let mut scene = ctx.create_scene("main");

            scene.spawn((
                Transform::default()
            ));
        }
    );

    runtime.on_frame(
        |mut ctx: RuntimeContext, delta: f32| {
        }
    );

    runtime.start();
}
