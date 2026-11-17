use crate::ecs::Commands;

pub struct Entity {}

impl Entity {
    pub fn new() -> () {
        Commands::spawn();
    }
}
