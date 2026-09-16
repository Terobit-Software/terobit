#[path = "listeners/window_listener.rs"]
mod window_listener;

#[cfg(test)]
mod tests {
    use runtime::listeners::{Listener, ListenerValue, WindowListener};

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
        }
    }
}
