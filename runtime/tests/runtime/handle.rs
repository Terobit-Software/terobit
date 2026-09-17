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
}
