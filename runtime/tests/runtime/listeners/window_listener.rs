#[cfg(test)]
mod tests {
    use runtime::listeners::{Listener, WindowListener};

    struct DefaultListener;

    impl Listener for DefaultListener {
        fn name(&self) -> &str {
            "default"
        }

        fn priority(&self) -> u8 {
            0
        }
    }

    impl WindowListener for DefaultListener {}

    #[test]
    fn implementors_can_rely_on_default_no_op_callbacks() {
        // `WindowListener`'s callbacks all have default (no-op) bodies, so a
        // listener only needs to implement the `Listener` supertrait to compile.
        let listener = DefaultListener;

        assert_eq!(listener.name(), "default");
        assert_eq!(listener.priority(), 0);
    }

    #[test]
    fn boxed_as_a_trait_object() {
        let listener: Box<dyn WindowListener> = Box::new(DefaultListener);

        assert_eq!(listener.name(), "default");
    }
}
