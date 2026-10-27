use terobit_v1_macros::{main, update};

#[main]
pub fn entry() {
    println!("test");
}

#[update]
pub fn update() {
    println!("update")
}
