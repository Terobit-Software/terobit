#[path = "listeners/window_listener.rs"]
mod window_listener;
#[path = "listeners/input_listener.rs"]
mod input_listener;

#[cfg(test)]
mod tests {
    use runtime::handle::RuntimeHandle;
    use runtime::listeners::{
        InputListener, KeyInput, Listener, ListenerValue, MouseInput, WindowListener,
    };

    struct MockListener {
        priority: u8,
        name: &'static str,
    }

    impl Listener for MockListener {
        fn name(&self) -> &str {
            self.name
        }

        fn priority(&self) -> u8 {
            self.priority
        }
    }

    impl WindowListener for MockListener {}

    struct MockInputListener {
        priority: u8,
        name: &'static str,
    }

    impl Listener for MockInputListener {
        fn name(&self) -> &str {
            self.name
        }

        fn priority(&self) -> u8 {
            self.priority
        }
    }

    impl InputListener for MockInputListener {
        fn on_key_input(&mut self, _input: KeyInput, _handle: &RuntimeHandle) {}
        fn on_mouse_input(&mut self, _input: MouseInput, _handle: &RuntimeHandle) {}
    }

    #[test]
    fn reports_its_own_priority_and_name() {
        let listener = MockListener {
            priority: 4,
            name: "mock",
        };

        assert_eq!(listener.priority(), 4);
        assert_eq!(listener.name(), "mock");
    }

    #[test]
    fn boxed_window_listener_wraps_into_a_window_listener_value() {
        let listener: Box<dyn WindowListener> = Box::new(MockListener {
            priority: 1,
            name: "boxed",
        });

        let value = ListenerValue::Window(listener);

        match value {
            ListenerValue::Window(inner) => assert_eq!(inner.name(), "boxed"),
            ListenerValue::Input(_) => panic!("expected window listener"),
        }
    }

    #[test]
    fn boxed_input_listener_wraps_into_an_input_listener_value() {
        let listener: Box<dyn InputListener> = Box::new(MockInputListener {
            priority: 1,
            name: "boxed_input",
        });

        let value = ListenerValue::Input(listener);

        match value {
            ListenerValue::Input(inner) => assert_eq!(inner.name(), "boxed_input"),
            ListenerValue::Window(_) => panic!("expected input listener"),
        }
    }
}
