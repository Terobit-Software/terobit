#[path = "runtime/handle.rs"]
mod handle;
#[path = "runtime/listeners.rs"]
mod listeners;
#[path = "runtime/routines.rs"]
mod routines;

#[cfg(test)]
mod tests {
    use runtime::handle::RuntimeHandle;
    use runtime::routines::RuntimeRoutine;
    use runtime::{Runtime, RuntimeInitError};

    struct NoOpRoutine;

    impl RuntimeRoutine for NoOpRoutine {
        fn priority(&self) -> u8 {
            0
        }

        fn frame(&self, _handle: &RuntimeHandle) {}

        fn name(&self) -> &str {
            "no_op"
        }

        fn is_enabled(&self) -> bool {
            true
        }
    }

    #[test]
    fn new_creates_a_runtime() {
        let _runtime = Runtime::new();
    }

    #[test]
    fn add_routine_before_start_does_not_panic() {
        let mut runtime = Runtime::new();
        runtime.add_routine(NoOpRoutine);
        runtime.add_routine(NoOpRoutine);
    }

    #[test]
    fn init_error_messages() {
        assert_eq!(
            RuntimeInitError::UnknownInitFail.to_string(),
            "failed to initialize"
        );
        assert_eq!(RuntimeInitError::WgpuFailed.to_string(), "wgpu failed");
        assert_eq!(
            RuntimeInitError::WinitEventLoopFailed.to_string(),
            "winit event loop failed"
        );
        assert_eq!(
            RuntimeInitError::FailedToAddRoutine.to_string(),
            "failed to add routine"
        );
    }

    #[test]
    fn init_error_implements_debug() {
        assert_eq!(format!("{:?}", RuntimeInitError::WgpuFailed), "WgpuFailed");
    }
}
