

/// simply a marker for now, probably add tweakable settings later from the component
/// requires a transform
#[derive(bevy_ecs::component::Component, Default)]
pub struct Camera {}

impl Camera {
    pub fn new () -> Self {
        Camera {}
    }
}