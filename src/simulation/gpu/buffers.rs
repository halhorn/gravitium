use bevy::asset::RenderAssetUsages;
use bevy::prelude::{Assets, Handle, Resource, Vec4};
use bevy::render::{
    extract_resource::ExtractResource, render_resource::BufferUsages, storage::ShaderStorageBuffer,
};
use bytemuck::Pod;

use crate::model::constants::{BODY_COUNT, MERGE_BUCKET_COUNT};
use super::params::MERGE_SCRATCH_LEN;

/// GPU-resident simulation buffers (handles shared by compute + render).
#[derive(Resource, Clone, ExtractResource)]
pub struct SimulationGpuBuffers {
    pub positions: Handle<ShaderStorageBuffer>,
    pub velocities: Handle<ShaderStorageBuffer>,
    pub masses: Handle<ShaderStorageBuffer>,
    pub accelerations: Handle<ShaderStorageBuffer>,
    pub accelerations_new: Handle<ShaderStorageBuffer>,
    /// Merge pass: spatial hash bucket heads (`u32::MAX` = empty).
    pub merge_bucket_heads: Handle<ShaderStorageBuffer>,
    /// Merge pass: `[0..n)` bucket_next, `[n..2n)` merge flash (see merge.wgsl).
    pub merge_aux: Handle<ShaderStorageBuffer>,
    /// Merge pass: smallest survivor index `i` per absorbed target `j`.
    pub merge_owner: Handle<ShaderStorageBuffer>,
    /// Merge snapshot: `[0..n)` pos.xyz + mass in w, `[n..2n)` velocity.
    pub merge_scratch: Handle<ShaderStorageBuffer>,
    /// Per-body base RGB (mass gradient + merge flash); lighting applied in the vertex shader.
    pub body_colors: Handle<ShaderStorageBuffer>,
}

impl SimulationGpuBuffers {
    fn init_storage_buffer<T: Pod>(
        buffers: &mut Assets<ShaderStorageBuffer>,
        data: Vec<T>,
        usage: BufferUsages,
        asset_usage: RenderAssetUsages,
    ) -> Handle<ShaderStorageBuffer> {
        let mut buffer =
            ShaderStorageBuffer::new(bytemuck::cast_slice(data.as_slice()), asset_usage);
        buffer.buffer_description.usage = usage;
        buffers.add(buffer)
    }

    pub fn new(
        buffers: &mut Assets<ShaderStorageBuffer>,
        positions: Vec<Vec4>,
        velocities: Vec<Vec4>,
        masses: Vec<f32>,
        accelerations: Vec<Vec4>,
    ) -> Self {
        debug_assert_eq!(positions.len(), BODY_COUNT);
        let usage = BufferUsages::STORAGE | BufferUsages::COPY_DST;
        let asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;

        let readback_usage = usage | BufferUsages::COPY_SRC;
        let positions = Self::init_storage_buffer(buffers, positions, readback_usage, asset_usage);
        let velocities = Self::init_storage_buffer(buffers, velocities, usage, asset_usage);
        let masses = Self::init_storage_buffer(buffers, masses, readback_usage, asset_usage);
        let accelerations = Self::init_storage_buffer(buffers, accelerations, usage, asset_usage);
        let accelerations_new = Self::init_storage_buffer(
            buffers,
            vec![Vec4::ZERO; BODY_COUNT],
            usage,
            asset_usage,
        );
        let merge_bucket_heads = Self::init_storage_buffer(
            buffers,
            vec![u32::MAX; MERGE_BUCKET_COUNT],
            usage,
            asset_usage,
        );
        // merge_aux layout is `[0..n) bucket_next, [n..2n) merge_flash` (n = active_count).
        // Zero-init so the colors pass does not treat stale slots as merge flash before the
        // first merge prepare run (flash region moves with n, so do not zero only [BODY_COUNT..)).
        let merge_aux = Self::init_storage_buffer(
            buffers,
            vec![0u32; BODY_COUNT * 2],
            usage,
            asset_usage,
        );
        let merge_owner = Self::init_storage_buffer(
            buffers,
            vec![BODY_COUNT as u32; BODY_COUNT],
            usage,
            asset_usage,
        );
        let merge_scratch = Self::init_storage_buffer(
            buffers,
            vec![Vec4::ZERO; MERGE_SCRATCH_LEN],
            usage,
            asset_usage,
        );
        let body_colors = Self::init_storage_buffer(
            buffers,
            vec![Vec4::ZERO; BODY_COUNT],
            usage,
            asset_usage,
        );

        Self {
            positions,
            velocities,
            masses,
            accelerations,
            accelerations_new,
            merge_bucket_heads,
            merge_aux,
            merge_owner,
            merge_scratch,
            body_colors,
        }
    }
}
