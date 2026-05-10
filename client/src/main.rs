use runtime::{Runtime, RuntimeContext};
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
            ctx.create_scene("scene or sum shi");
        }
    );

    runtime.on_frame(
        |mut ctx: RuntimeContext, delta: f32| {
            println!("{}", rand::random::<u32>())
        }
    );

    runtime.start();
}
