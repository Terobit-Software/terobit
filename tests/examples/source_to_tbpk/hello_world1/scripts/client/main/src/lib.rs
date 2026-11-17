use terobit_v1::{ecs::Entity, prelude::*};

#[main]
fn main() {
    log("yuh");
}

#[update::post_render]
fn spawn() {
    Entity::new();
}
