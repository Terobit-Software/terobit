#[cfg(test)]
mod tests {
    use runtime::handle::RuntimeHandle;
    use runtime::listeners::{
        InputListener, KeyInput, Listener, MouseButton, MouseInput, MouseInputType,
    };

    struct DefaultListener;

    impl Listener for DefaultListener {
        fn name(&self) -> &str {
            "default_input"
        }

        fn priority(&self) -> u8 {
            0
        }
    }

    impl InputListener for DefaultListener {
        fn on_key_input(&mut self, _input: KeyInput, _handle: &RuntimeHandle) {}
        fn on_mouse_input(&mut self, _input: MouseInput, _handle: &RuntimeHandle) {}
    }

    #[test]
    fn implementors_must_provide_both_callbacks_to_compile() {
        // `InputListener`'s callbacks have no default bodies (unlike `WindowListener`'s),
        // so a listener must implement both to satisfy the trait.
        let listener = DefaultListener;

        assert_eq!(listener.name(), "default_input");
        assert_eq!(listener.priority(), 0);
    }

    #[test]
    fn boxed_as_a_trait_object() {
        let listener: Box<dyn InputListener> = Box::new(DefaultListener);

        assert_eq!(listener.name(), "default_input");
    }

    #[test]
    fn key_input_is_copy() {
        let input = KeyInput::KeyPressed('a');
        let copy = input;

        // `input` still usable after `copy` was created confirms `Copy`, not just `Clone`.
        assert_eq!(format!("{:?}", input), format!("{:?}", copy));
    }

    #[test]
    fn mouse_input_is_copy_and_exposes_its_fields() {
        let input = MouseInput {
            input_type: MouseInputType::Move,
            x: 1.5,
            y: -2.5,
            button: MouseButton::None,
        };
        let copy = input;

        assert_eq!(copy.x, 1.5);
        assert_eq!(copy.y, -2.5);
        assert_eq!(format!("{:?}", input.input_type), "Move");
        assert_eq!(format!("{:?}", input.button), "None");
    }
}
