use terobit_v1::prelude::*;

#[main]
fn main() {
    log("yuh");
}

#[update::pre_render]
fn update() {
    log("yee");
    let q = Query::<With<Transform>>::query();
}
