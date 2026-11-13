use core::marker::PhantomData;

use crate::ecs::components::{ComponentId, Components};
use crate::ecs::{Read, With, Write};

/// What a query needs from a set of components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    With,
    Read,
    Write,
}

/// One part of a query filter, e.g. `Write<Mesh>` -> `{ Write, [Mesh::ID] }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Term {
    pub access: Access,
    pub components: &'static [ComponentId],
}

/// A single modifier: `With<T>`, `Read<T>` or `Write<T>`.
pub trait QueryTerm {
    const TERM: Term;
}

impl<T: Components> QueryTerm for With<T> {
    const TERM: Term = Term {
        access: Access::With,
        components: T::IDS,
    };
}

impl<T: Components> QueryTerm for Read<T> {
    const TERM: Term = Term {
        access: Access::Read,
        components: T::IDS,
    };
}

impl<T: Components> QueryTerm for Write<T> {
    const TERM: Term = Term {
        access: Access::Write,
        components: T::IDS,
    };
}

/// One modifier or a tuple of modifiers, e.g. `(With<(Name, Transform)>, Write<Mesh>)`.
pub trait QueryFilter {
    const TERMS: &'static [Term];
}

impl<T: QueryTerm> QueryFilter for T {
    const TERMS: &'static [Term] = &[T::TERM];
}

macro_rules! impl_query_filter_for_tuple {
    ($($t:ident),+) => {
        impl<$($t: QueryTerm),+> QueryFilter for ($($t,)+) {
            const TERMS: &'static [Term] = &[$($t::TERM),+];
        }
    };
}

impl_query_filter_for_tuple!(A);
impl_query_filter_for_tuple!(A, B);
impl_query_filter_for_tuple!(A, B, C);
impl_query_filter_for_tuple!(A, B, C, D);
impl_query_filter_for_tuple!(A, B, C, D, E);
impl_query_filter_for_tuple!(A, B, C, D, E, F);
impl_query_filter_for_tuple!(A, B, C, D, E, F, G);
impl_query_filter_for_tuple!(A, B, C, D, E, F, G, H);

/// A query over entities matching `F`.
///
/// ```rust,ignore
/// let q = Query::<(With<(Name, Transform)>, Write<Mesh>)>::query();
/// ```
pub struct Query<F: QueryFilter>(PhantomData<F>);

impl<F: QueryFilter> Query<F> {
    /// The terms this query was built from, in the order they were written.
    pub const TERMS: &'static [Term] = F::TERMS;

    // TODO: call the host once the ECS ABI exists; for now this only
    // resolves the filter.
    pub fn query() -> Self {
        Self(PhantomData)
    }

    pub fn terms(&self) -> &'static [Term] {
        F::TERMS
    }
}
