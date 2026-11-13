pub type ComponentId = u32;

/// A type that can be stored on an entity and used in queries.
pub trait Component {
    const ID: ComponentId;
}

/// One component or a tuple of components, e.g. `Transform` or `(Transform, Name)`.
pub trait Components {
    const IDS: &'static [ComponentId];
}

impl<T: Component> Components for T {
    const IDS: &'static [ComponentId] = &[T::ID];
}

macro_rules! impl_components_for_tuple {
    ($($t:ident),+) => {
        impl<$($t: Component),+> Components for ($($t,)+) {
            const IDS: &'static [ComponentId] = &[$($t::ID),+];
        }
    };
}

impl_components_for_tuple!(A);
impl_components_for_tuple!(A, B);
impl_components_for_tuple!(A, B, C);
impl_components_for_tuple!(A, B, C, D);
impl_components_for_tuple!(A, B, C, D, E);
impl_components_for_tuple!(A, B, C, D, E, F);
impl_components_for_tuple!(A, B, C, D, E, F, G);
impl_components_for_tuple!(A, B, C, D, E, F, G, H);

mod transform;
pub use transform::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::With;

    struct Name;
    impl Component for Name {
        const ID: ComponentId = 2;
    }

    #[test]
    fn with_single_and_tuple() {
        assert_eq!(With::<Transform>::COMPONENTS, &[Transform::ID]);
        assert_eq!(
            With::<(Transform, Name)>::COMPONENTS,
            &[Transform::ID, Name::ID]
        );
    }
}
