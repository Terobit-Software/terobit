use runtime::ecs::Mesh;
use runtime::handle::RuntimeHandle;
use runtime::listeners::WindowListener;

pub struct ClientWindowListener {}

impl WindowListener for ClientWindowListener {
    fn on_window_create(&self, _handle: &RuntimeHandle) {
        _handle.world().spawn(Mesh::new(0));
    }
    fn priority(&self) -> u32 {
        0
    }
}
