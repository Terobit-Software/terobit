use terobit_v1::{ecs::Entity, prelude::*};

#[main]
fn main() {
    log("yuh");
}

#[update::post_render]
fn spawn() {
    for i in 0..1000 {
        Entity::new().with(Transform::new()).spawn();
    }
    set_title("test title");
    log(&format!("fps: {:#?}", 1.0 / FrameInfo::get().delta_secs()));
}
