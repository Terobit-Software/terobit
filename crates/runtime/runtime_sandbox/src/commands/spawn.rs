use crate::commands::GuestCommand;
use bevy::{ecs::entity::Entity, prelude::Commands};

pub fn command(commands: &mut Commands, command: &GuestCommand) -> Entity {
    if command.payload.len() == 0 {
        return commands.spawn_empty().id();
    } else {
        // TODO: finish this! decode and spawn with the components requested
        return commands.spawn(()).id();
    };
}
