use terobit_v1::{ecs::Entity, prelude::*};

#[main]
fn main() {
    log("yuh");
}

fn print_fps() {
    log(&format!("fps: {:#?}", 1.0 / FrameInfo::get().delta_secs()));
}

#[update::post_render]
fn spawn() {
    // add query code

    print_fps();
}
