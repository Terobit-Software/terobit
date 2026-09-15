// Everything under `runtime::app` is `pub(crate)`, so these submodules have
// nothing to import from an external test crate; they stay as placeholders.
#[path = "app/app.rs"]
mod app;
#[path = "app/instance.rs"]
mod instance;
#[path = "app/mesh.rs"]
mod mesh;
#[path = "app/state.rs"]
mod state;
#[path = "app/window.rs"]
mod window;
