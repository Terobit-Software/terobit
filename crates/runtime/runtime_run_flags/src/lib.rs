use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct RuntimeFlags {
    flags: HashMap<String, Vec<String>>,
}

impl RuntimeFlags {
    pub fn read() -> Self {
        Self::from_args(std::env::args().skip(1))
    }

    pub fn get_flags(&self) -> &HashMap<String, Vec<String>> {
        &self.flags
    }

    pub fn from_args(args: impl IntoIterator<Item = String>) -> Self {
        let mut flags = Self::default();

        let mut stack: Vec<String> = Vec::new();

        for arg in args {
            if arg.starts_with("--") {
                if let Some((flag, values)) = stack.split_first() {
                    flags.flags.insert(flag.clone(), values.to_vec());
                }
                stack.clear();
                stack.push(arg);
                continue;
            }

            // values before the first flag don't belong to anything
            if !stack.is_empty() {
                stack.push(arg);
            }
        }

        if let Some((flag, values)) = stack.split_first() {
            flags.flags.insert(flag.clone(), values.to_vec());
        }

        flags
    }
}

pub struct RuntimeRunFlagsPlugin;

impl Plugin for RuntimeRunFlagsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RuntimeFlags::read());
    }
}

#[cfg(test)]
mod struct_tests {
    use super::*;

    fn args(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn runtime_args_read_correctly() {
        let flags = RuntimeFlags::from_args(args(&[
            "--pack",
            "game.pack",
            "--mods",
            "a",
            "b",
            "--verbose",
        ]));

        assert_eq!(flags.flags.get("--pack"), Some(&args(&["game.pack"])));
        assert_eq!(flags.flags.get("--mods"), Some(&args(&["a", "b"])));
        assert_eq!(flags.flags.get("--verbose"), Some(&args(&[])));
        assert_eq!(flags.flags.len(), 3);
    }

    #[test]
    fn runtime_args_ignore_values_before_first_flag() {
        let flags = RuntimeFlags::from_args(args(&["test", "test", "--value", "number"]));

        assert_eq!(flags.flags.get("--value"), Some(&args(&["number"])));
        assert_eq!(flags.flags.len(), 1);
    }

    #[test]
    fn runtime_args_empty() {
        let flags = RuntimeFlags::from_args(args(&[]));

        assert!(flags.flags.is_empty());
    }
}

#[cfg(test)]
mod plugin_tests {
    use super::*;

    #[test]
    fn adds_resource() {
        let mut app = App::new();
        app.add_plugins(RuntimeRunFlagsPlugin);

        assert!(app.world().get_resource::<RuntimeFlags>().is_some());
    }

    #[test]
    fn resource_matches_process_args() {
        let mut app = App::new();
        app.add_plugins(RuntimeRunFlagsPlugin);

        let flags = app.world().resource::<RuntimeFlags>();
        assert_eq!(flags.flags, RuntimeFlags::read().flags);
    }
}
