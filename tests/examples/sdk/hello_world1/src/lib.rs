use terobit_v1::log;
use terobit_v1::main;
use terobit_v1::update;

#[main]
fn main() {
    log("yuh");
}

#[update::pre_render]
fn update() {
    log("yee");
}
