#[cfg(test)]
mod tests {
    use runtime::handle::RuntimeCommand;

    #[test]
    fn terminate_command_equals_itself() {
        assert_eq!(RuntimeCommand::Terminate, RuntimeCommand::Terminate);
    }

    #[test]
    fn terminate_command_is_clone() {
        let original = RuntimeCommand::Terminate;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn terminate_command_debug_format() {
        assert_eq!(format!("{:?}", RuntimeCommand::Terminate), "Terminate");
    }

    #[test]
    fn set_window_title_commands_with_equal_titles_are_equal() {
        assert_eq!(
            RuntimeCommand::SetWindowTitle("foo".to_string()),
            RuntimeCommand::SetWindowTitle("foo".to_string())
        );
    }

    #[test]
    fn set_window_title_commands_with_different_titles_are_not_equal() {
        assert_ne!(
            RuntimeCommand::SetWindowTitle("foo".to_string()),
            RuntimeCommand::SetWindowTitle("bar".to_string())
        );
    }

    #[test]
    fn terminate_and_set_window_title_are_not_equal() {
        assert_ne!(
            RuntimeCommand::Terminate,
            RuntimeCommand::SetWindowTitle("foo".to_string())
        );
    }

    #[test]
    fn set_window_title_command_is_clone() {
        let original = RuntimeCommand::SetWindowTitle("foo".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn set_window_title_command_debug_format() {
        assert_eq!(
            format!("{:?}", RuntimeCommand::SetWindowTitle("foo".to_string())),
            "SetWindowTitle(\"foo\")"
        );
    }
}
