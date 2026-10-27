use terobit_v1::main;
use terobit_v1::update;

#[main]
fn main() {
    println!("yuh");
}

#[update::pre_render]
fn update() {
    println!("yee");
}
