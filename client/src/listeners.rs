use rand::RngExt;
use runtime::ecs::{Mesh, Transform};
use runtime::handle::RuntimeHandle;
use runtime::listeners::{InputListener, Listener, WindowListener};
use runtime::listeners::{KeyInput, MouseInput};

pub struct ClientWindowListener {}

impl WindowListener for ClientWindowListener {
    fn on_window_create(&self, _handle: &RuntimeHandle) {
        let mut rng = rand::rng();
        // let num: u8 = rng.random();
        //_handle.set_title(&num.to_string());
        // if num == 0 {
        // scatter cubes around the origin, each at a random orientation so
        // they read as 3D
        for _ in 0..2_000_000 {
            let transform = Transform::new()
                .with_pos(glam::Vec3::new(
                    rng.random_range(-100.0..100.0),
                    rng.random_range(-100.0..100.0),
                    rng.random_range(-100.0..100.0),
                ))
                .with_rot(glam::Quat::from_euler(
                    glam::EulerRot::XYZ,
                    rng.random_range(0.0..std::f32::consts::TAU),
                    rng.random_range(0.0..std::f32::consts::TAU),
                    rng.random_range(0.0..std::f32::consts::TAU),
                ))
                .with_scale(glam::Vec3::splat(0.3));
            _handle.world().spawn((Mesh::CUBE, transform));
        }
    }
}

impl Listener for ClientWindowListener {
    fn name(&self) -> &str {
        "main_window_listener"
    }
    fn priority(&self) -> u8 {
        0
    }
}

pub struct ClientInputListener {}

impl Listener for ClientInputListener {
    fn name(&self) -> &str {
        "client_input_listener"
    }
    fn priority(&self) -> u8 {
        0
    }
}

impl InputListener for ClientInputListener {
    fn on_key_input(&mut self, input: KeyInput, _handle: &RuntimeHandle) {
        // println!("{:?}", input)
    }
    fn on_mouse_input(&mut self, input: MouseInput, _handle: &RuntimeHandle) {
        // println!("{:?}", input)
    }
}
