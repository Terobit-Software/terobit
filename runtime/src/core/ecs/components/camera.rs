/// marker component for the camera. requires a `Transform` on the same entity.
/// the renderer uses the first entity with `(Transform, Camera)` as the active camera.
/// will probably get actual settings later
#[derive(bevy_ecs::component::Component, Default)]
pub struct Camera {}

impl Camera {
    /// creates a new camera marker
    pub fn new () -> Self {
        Camera {}
    }
}