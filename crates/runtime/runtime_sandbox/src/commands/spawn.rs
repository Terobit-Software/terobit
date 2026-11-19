use crate::commands::{GuestCommand, util::decode_components::insert_components};
use bevy::{ecs::entity::Entity, prelude::Commands};

pub fn command(commands: &mut Commands, command: &GuestCommand) -> Entity {
    let mut entity = commands.spawn_empty();

    if let Err(e) = insert_components(&mut entity, &command.payload) {
        tracing::error!(
            "Failed to decode spawn components: {}",
            snafu::Report::from_error(&e)
        );
    }

    entity.id()
}
