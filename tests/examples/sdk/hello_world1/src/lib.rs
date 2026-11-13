use terobit_v1::prelude::*;

#[main]
fn main() {
    log("yuh");
}

#[update::pre_render]
fn update() {
    log("yee");
}

#[update::post_render]
fn spawn() {
    Commands::spawn();
}
