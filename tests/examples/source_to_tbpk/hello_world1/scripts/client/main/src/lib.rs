use terobit_v1::{ecs::Command, prelude::*};

#[main]
fn main() {
    log("yuh");
}

#[update::post_render]
fn spawn() {
    log("spawning spawn command");
    Commands::spawn();
}
