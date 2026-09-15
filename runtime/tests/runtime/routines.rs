#[cfg(test)]
mod tests {
    use runtime::handle::RuntimeHandle;
    use runtime::routines::RuntimeRoutine;

    struct MockRoutine {
        priority: u8,
        name: &'static str,
        enabled: bool,
    }

    impl RuntimeRoutine for MockRoutine {
        fn priority(&self) -> u8 {
            self.priority
        }

        fn frame(&self, _handle: &RuntimeHandle) {}

        fn name(&self) -> &str {
            self.name
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }

    #[test]
    fn reports_its_own_priority_name_and_enabled_state() {
        let routine = MockRoutine {
            priority: 5,
            name: "mock",
            enabled: false,
        };

        assert_eq!(routine.priority(), 5);
        assert_eq!(routine.name(), "mock");
        assert!(!routine.is_enabled());
    }

    #[test]
    fn boxed_routines_sort_by_priority_lowest_first() {
        let mut routines: Vec<Box<dyn RuntimeRoutine>> = vec![
            Box::new(MockRoutine {
                priority: 3,
                name: "c",
                enabled: true,
            }),
            Box::new(MockRoutine {
                priority: 1,
                name: "a",
                enabled: true,
            }),
            Box::new(MockRoutine {
                priority: 2,
                name: "b",
                enabled: true,
            }),
        ];

        routines.sort_by_key(|routine| routine.priority());

        let names: Vec<&str> = routines.iter().map(|routine| routine.name()).collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }
}
