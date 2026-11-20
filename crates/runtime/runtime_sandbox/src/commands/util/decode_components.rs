//! Decodes the component list a guest attaches to a command payload
//! (see `terobit_v1::ecs::commands::Spawn`).
//!
//! The list is a flat run of records, each laid out as:
//!
//! | bytes          | field                     |
//! |----------------|---------------------------|
//! | `u32` (LE)     | component id              |
//! | `u32` (LE)     | component data length     |
//! | `[u8; length]` | component data            |

use bevy::{
    ecs::system::EntityCommands,
    math::{Quat, Vec3},
    transform::components::Transform,
};
use snafu::{OptionExt, Snafu};
use terobit_v1_abi::ecs::components::transform::{self, TRANSFORM_COMPONENT_ID};

/// Size of a component record header (id + data length).
const RECORD_HEADER_LEN: usize = 8;

#[derive(Debug, Snafu)]
pub enum DecodeComponentsError {
    #[snafu(display("malformed component record at byte {offset}"))]
    Malformed { offset: usize },

    #[snafu(display("component {id} expected {expected} bytes, got {got}"))]
    WrongSize {
        id: u32,
        expected: usize,
        got: usize,
    },
}

/// Decodes every component in `payload` and inserts it onto `entity`.
///
/// Unknown component ids are skipped with a warning, so a guest built against
/// a newer SDK still runs. Stops at the first malformed record; components
/// before it have already been inserted.
pub fn insert_components(
    entity: &mut EntityCommands,
    mut payload: &[u8],
) -> Result<(), DecodeComponentsError> {
    let total = payload.len();

    while !payload.is_empty() {
        let offset = total - payload.len();

        let (header, rest) = payload
            .split_first_chunk::<RECORD_HEADER_LEN>()
            .context(MalformedSnafu { offset })?;
        let id = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let len = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;

        let (data, rest) = rest
            .split_at_checked(len)
            .context(MalformedSnafu { offset })?;

        insert_component(entity, id, data)?;
        payload = rest;
    }

    Ok(())
}

/// Decodes one component and inserts it. Add new components here.
fn insert_component(
    entity: &mut EntityCommands,
    id: u32,
    data: &[u8],
) -> Result<(), DecodeComponentsError> {
    match id {
        TRANSFORM_COMPONENT_ID => {
            let t: transform::Transform = read(id, data)?;
            entity.insert(Transform {
                translation: Vec3::from_array(t.translation),
                rotation: Quat::from_array(t.rotation),
                scale: Vec3::from_array(t.scale),
            });
        }
        id => tracing::warn!(
            "unknown guest component id {} ({} bytes), skipping",
            id,
            data.len()
        ),
    }

    Ok(())
}

/// Reads an ABI component out of `data`, which must be exactly its size.
fn read<T: bytemuck::Pod>(id: u32, data: &[u8]) -> Result<T, DecodeComponentsError> {
    bytemuck::try_pod_read_unaligned(data)
        .ok()
        .context(WrongSizeSnafu {
            id,
            expected: size_of::<T>(),
            got: data.len(),
        })
}
