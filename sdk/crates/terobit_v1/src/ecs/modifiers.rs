use crate::ecs::components::Component;

/// With

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct With<const C: Component>;

impl<const C: Component> With<C> {
    /// The component ID this filter requires.
    pub const COMPONENT: Component = C;
}

/// Read

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Read<const C: Component>;

impl<const C: Component> Read<C> {
    /// The component ID this filter requires.
    pub const COMPONENT: Component = C;
}

/// Write

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Write<const C: Component>;

impl<const C: Component> Write<C> {
    /// The component ID this filter requires.
    pub const COMPONENT: Component = C;
}
