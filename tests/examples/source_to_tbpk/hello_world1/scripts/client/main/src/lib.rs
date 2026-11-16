use terobit_v1::prelude::*;

#[main]
fn main() {
    log("yuh");
}

#[update::pre_render]
fn update() {
    log("yeea");
}

#[update::post_render]
fn spawn() {
    Commands::spawn();
}
