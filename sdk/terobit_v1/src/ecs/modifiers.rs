use core::marker::PhantomData;

use crate::ecs::components::{ComponentId, Components};

macro_rules! modifier {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        pub struct $name<T: Components>(PhantomData<T>);

        impl<T: Components> $name<T> {
            /// The component IDs this modifier applies to.
            pub const COMPONENTS: &'static [ComponentId] = T::IDS;
        }
    };
}

modifier!(
    /// Only match entities that have `T`.
    With
);
modifier!(
    /// Read access to `T`.
    Read
);
modifier!(
    /// Write access to `T`.
    Write
);
