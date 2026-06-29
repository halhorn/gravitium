use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::render_resource::{
        binding_types::{storage_buffer, storage_buffer_read_only, uniform_buffer},
        *,
    },
};

use crate::simulation::shaders::{GRAVITY_SHADER, INTEGRATE_SHADER, MERGE_SHADER, COLORS_SHADER};

use super::params::{ColorsParams, GravityParams, IntegrateParams, MergeParams};

#[derive(Resource)]
pub struct SimulationComputePipelines {
    pub gravity_layout: BindGroupLayoutDescriptor,
    pub integrate_layout: BindGroupLayoutDescriptor,
    pub merge_layout: BindGroupLayoutDescriptor,
    pub colors_layout: BindGroupLayoutDescriptor,
    pub gravity: CachedComputePipelineId,
    pub position_step: CachedComputePipelineId,
    pub velocity_step: CachedComputePipelineId,
    pub merge_prepare: CachedComputePipelineId,
    pub merge_finalize_cell_size: CachedComputePipelineId,
    pub merge_clear_buckets: CachedComputePipelineId,
    pub merge_init_owner: CachedComputePipelineId,
    pub merge_build_grid: CachedComputePipelineId,
    pub merge_find_owner: CachedComputePipelineId,
    pub merge_apply: CachedComputePipelineId,
    pub colors: CachedComputePipelineId,
}

pub fn init_simulation_compute_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
) {
    let gravity_layout = BindGroupLayoutDescriptor::new(
        "gravity_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer_read_only::<Vec<Vec4>>(false),
                storage_buffer_read_only::<Vec<f32>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                uniform_buffer::<GravityParams>(false),
            ),
        ),
    );

    let integrate_layout = BindGroupLayoutDescriptor::new(
        "integrate_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer_read_only::<Vec<Vec4>>(false),
                storage_buffer_read_only::<Vec<f32>>(false),
                uniform_buffer::<IntegrateParams>(false),
            ),
        ),
    );

    let merge_layout = BindGroupLayoutDescriptor::new(
        "merge_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer::<Vec<f32>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                storage_buffer::<Vec<u32>>(false),
                storage_buffer::<Vec<u32>>(false),
                storage_buffer::<Vec<u32>>(false),
                uniform_buffer::<MergeParams>(false),
            ),
        ),
    );

    let colors_layout = BindGroupLayoutDescriptor::new(
        "colors_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer_read_only::<Vec<f32>>(false),
                storage_buffer_read_only::<Vec<u32>>(false),
                storage_buffer::<Vec<Vec4>>(false),
                uniform_buffer::<ColorsParams>(false),
            ),
        ),
    );

    let gravity = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("gravity".into()),
        layout: vec![gravity_layout.clone()],
        shader: GRAVITY_SHADER.clone(),
        entry_point: Some(Cow::from("main")),
        ..default()
    });

    let position_step = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("position_step".into()),
        layout: vec![integrate_layout.clone()],
        shader: INTEGRATE_SHADER.clone(),
        entry_point: Some(Cow::from("position_step")),
        ..default()
    });

    let velocity_step = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("velocity_step".into()),
        layout: vec![integrate_layout.clone()],
        shader: INTEGRATE_SHADER,
        entry_point: Some(Cow::from("velocity_step")),
        ..default()
    });

    let merge_prepare = queue_merge_pipeline(&pipeline_cache, &merge_layout, "prepare");
    let merge_finalize_cell_size =
        queue_merge_pipeline(&pipeline_cache, &merge_layout, "finalize_cell_size");
    let merge_clear_buckets =
        queue_merge_pipeline(&pipeline_cache, &merge_layout, "clear_buckets");
    let merge_init_owner = queue_merge_pipeline(&pipeline_cache, &merge_layout, "init_owner");
    let merge_build_grid = queue_merge_pipeline(&pipeline_cache, &merge_layout, "build_grid");
    let merge_find_owner = queue_merge_pipeline(&pipeline_cache, &merge_layout, "find_owner");
    let merge_apply = queue_merge_pipeline(&pipeline_cache, &merge_layout, "apply_merge");

    let colors = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("colors".into()),
        layout: vec![colors_layout.clone()],
        shader: COLORS_SHADER.clone(),
        entry_point: Some(Cow::from("main")),
        ..default()
    });

    commands.insert_resource(SimulationComputePipelines {
        gravity_layout,
        integrate_layout,
        merge_layout,
        colors_layout,
        gravity,
        position_step,
        velocity_step,
        merge_prepare,
        merge_finalize_cell_size,
        merge_clear_buckets,
        merge_init_owner,
        merge_build_grid,
        merge_find_owner,
        merge_apply,
        colors,
    });
}

fn queue_merge_pipeline(
    pipeline_cache: &PipelineCache,
    layout: &BindGroupLayoutDescriptor,
    entry: &'static str,
) -> CachedComputePipelineId {
    pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(entry.into()),
        layout: vec![layout.clone()],
        shader: MERGE_SHADER.clone(),
        entry_point: Some(Cow::from(entry)),
        ..default()
    })
}
