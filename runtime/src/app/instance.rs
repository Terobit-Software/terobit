use std::collections::HashMap;

use bevy_ecs::entity::Entity;
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::world::World;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use tracing::{debug, info};

use crate::assets::AssetId;
use crate::consts::render::INITIAL_INSTANCE_CAPACITY;
use crate::ecs::{Mesh, Transform};

/// Per-instance data uploaded alongside the shared cube mesh: just a model
/// matrix for now, one row per instance drawn.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub(crate) fn from_model(model: Mat4) -> Self {
        Self {
            model: model.to_cols_array_2d(),
        }
    }
}

/// Every instance of one mesh: a dense CPU mirror of its GPU instance buffer,
/// plus which entity owns each slot so removals can swap-remove.
struct MeshInstances {
    buffer: wgpu::Buffer,
    /// how many instances `buffer` has room for
    capacity: usize,
    data: Vec<InstanceRaw>,
    owners: Vec<Entity>,
    /// slots written on the CPU since the last upload
    dirty: Vec<u32>,
}

impl MeshInstances {
    fn new(device: &wgpu::Device, asset_id: AssetId) -> Self {
        Self {
            buffer: create_instance_buffer(device, asset_id, INITIAL_INSTANCE_CAPACITY),
            capacity: INITIAL_INSTANCE_CAPACITY,
            data: Vec::new(),
            owners: Vec::new(),
            dirty: Vec::new(),
        }
    }

    /// Writes the dirty slots to the GPU, one `write_buffer` per contiguous
    /// run. If the buffer is too small it's reallocated and fully rewritten.
    /// Returns how many instances were written.
    fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, asset_id: AssetId) -> usize {
        if self.dirty.is_empty() {
            return 0;
        }

        if self.data.len() > self.capacity {
            self.capacity = self.data.len().next_power_of_two();
            self.buffer = create_instance_buffer(device, asset_id, self.capacity);
            info!(
                "Grew mesh {} instance buffer to {} instances",
                asset_id, self.capacity
            );

            self.dirty.clear();
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
            return self.data.len();
        }

        self.dirty.sort_unstable();
        self.dirty.dedup();
        // swap-removes can leave slots past the end that no longer exist
        let len = self.data.len() as u32;
        self.dirty.retain(|&slot| slot < len);

        let mut written = 0;
        for run in self.dirty.chunk_by(|a, b| a + 1 == *b) {
            let start = run[0] as usize;
            let end = start + run.len();
            queue.write_buffer(
                &self.buffer,
                (start * size_of::<InstanceRaw>()) as wgpu::BufferAddress,
                bytemuck::cast_slice(&self.data[start..end]),
            );
            written += run.len();
        }
        self.dirty.clear();

        written
    }
}

/// Creates a vertex buffer big enough for `capacity` per-instance model matrices.
fn create_instance_buffer(
    device: &wgpu::Device,
    asset_id: AssetId,
    capacity: usize,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("mesh {asset_id} instance buffer")),
        size: (capacity * size_of::<InstanceRaw>()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// GPU-resident instance data for every `(Mesh, Transform)` entity, kept in
/// sync incrementally: each entity owns a slot in its mesh's instance buffer,
/// and only slots whose entity was added, changed, or removed get rewritten.
pub(crate) struct InstanceCache {
    meshes: HashMap<AssetId, MeshInstances>,
    /// which mesh and slot each tracked entity lives in
    slots: HashMap<Entity, (AssetId, u32)>,
    /// index of the world the cache currently mirrors
    world_index: Option<usize>,
}

impl InstanceCache {
    pub(crate) fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            slots: HashMap::new(),
            world_index: None,
        }
    }

    /// Brings the GPU instance buffers up to date with `world`, then clears
    /// its change trackers so the next sync only sees newer changes. Switching
    /// to a different world rebuilds everything from scratch.
    pub(crate) fn sync(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world_index: usize,
        world: &mut World,
    ) {
        if self.world_index != Some(world_index) {
            self.world_index = Some(world_index);
            self.slots.clear();
            for instances in self.meshes.values_mut() {
                instances.data.clear();
                instances.owners.clear();
                instances.dirty.clear();
            }

            let mut query = world.query::<(Entity, &Mesh, &Transform)>();
            for (entity, mesh, transform) in query.iter(world) {
                self.insert(device, entity, mesh.asset_id(), transform);
            }
        } else {
            // removals first, so an entity whose component was removed and
            // re-added this frame is re-inserted by the changed query below
            let removed: Vec<Entity> = world
                .removed::<Mesh>()
                .chain(world.removed::<Transform>())
                .collect();
            for entity in removed {
                self.remove(entity);
            }

            // Changed also matches newly added components
            let mut query = world.query_filtered::<
                (Entity, &Mesh, &Transform),
                Or<(Changed<Mesh>, Changed<Transform>)>,
            >();
            for (entity, mesh, transform) in query.iter(world) {
                self.upsert(device, entity, mesh.asset_id(), transform);
            }
        }

        world.clear_trackers();

        let mut written = 0;
        for (&asset_id, instances) in &mut self.meshes {
            written += instances.upload(device, queue, asset_id);
        }
        if written > 0 {
            debug!("Uploaded {} changed mesh instances to the GPU", written);
        }
    }

    fn insert(
        &mut self,
        device: &wgpu::Device,
        entity: Entity,
        asset_id: AssetId,
        transform: &Transform,
    ) {
        let instances = self
            .meshes
            .entry(asset_id)
            .or_insert_with(|| MeshInstances::new(device, asset_id));

        let slot = instances.data.len() as u32;
        instances
            .data
            .push(InstanceRaw::from_model(transform.to_matrix()));
        instances.owners.push(entity);
        instances.dirty.push(slot);
        self.slots.insert(entity, (asset_id, slot));
    }

    /// Updates `entity`'s slot in place, or moves/inserts it if its mesh changed
    /// or it isn't tracked yet.
    fn upsert(
        &mut self,
        device: &wgpu::Device,
        entity: Entity,
        asset_id: AssetId,
        transform: &Transform,
    ) {
        match self.slots.get(&entity) {
            Some(&(current, slot)) if current == asset_id => {
                let instances = self.meshes.get_mut(&asset_id).expect("tracked mesh");
                instances.data[slot as usize] = InstanceRaw::from_model(transform.to_matrix());
                instances.dirty.push(slot);
            }
            Some(_) => {
                self.remove(entity);
                self.insert(device, entity, asset_id, transform);
            }
            None => self.insert(device, entity, asset_id, transform),
        }
    }

    /// Frees `entity`'s slot by moving its mesh's last instance into it.
    fn remove(&mut self, entity: Entity) {
        let Some((asset_id, slot)) = self.slots.remove(&entity) else {
            return;
        };

        let instances = self.meshes.get_mut(&asset_id).expect("tracked mesh");
        let slot_index = slot as usize;
        instances.data.swap_remove(slot_index);
        instances.owners.swap_remove(slot_index);

        if let Some(&moved) = instances.owners.get(slot_index) {
            self.slots.insert(moved, (asset_id, slot));
            instances.dirty.push(slot);
        }
    }

    /// Every mesh with at least one instance: its asset id, instance buffer,
    /// and instance count.
    pub(crate) fn batches(&self) -> impl Iterator<Item = (AssetId, &wgpu::Buffer, u32)> {
        self.meshes
            .iter()
            .filter(|(_, instances)| !instances.data.is_empty())
            .map(|(&asset_id, instances)| {
                (asset_id, &instances.buffer, instances.data.len() as u32)
            })
    }

    /// Total number of tracked instances across every mesh.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.slots.len()
    }
}
