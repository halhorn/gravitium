use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        storage::GpuShaderStorageBuffer,
    },
};

use crate::simulation::config::SimulationConfig;
use crate::simulation::merge_cell::MergeCellCpuState;
use crate::simulation::settings::SimulationSettings;

use super::buffers::SimulationGpuBuffers;
use super::params::{ColorsParams, GravityParams, IntegrateParams, MergeParams};
use super::pipelines::SimulationComputePipelines;

#[derive(Resource)]
pub struct SimulationComputeBindGroups {
    pub gravity: BindGroup,
    pub integrate: BindGroup,
    pub merge: BindGroup,
    pub colors: BindGroup,
}

#[derive(Default)]
pub(crate) struct CachedBindGroupParams {
    last: Option<(GravityParams, IntegrateParams, MergeParams, ColorsParams)>,
}

#[derive(Default)]
pub(crate) struct SimulationComputeUniforms {
    gravity: Option<UniformBuffer<GravityParams>>,
    integrate: Option<UniformBuffer<IntegrateParams>>,
    merge: Option<UniformBuffer<MergeParams>>,
    colors: Option<UniformBuffer<ColorsParams>>,
}

fn current_params(
    settings: &SimulationSettings,
    config: &SimulationConfig,
    merge_cell: &MergeCellCpuState,
) -> (GravityParams, IntegrateParams, MergeParams, ColorsParams) {
    (
        GravityParams::from_settings(settings),
        IntegrateParams::from_settings(settings, config),
        MergeParams::from_settings(settings, merge_cell),
        ColorsParams::from_settings(settings),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_simulation_bind_groups(
    mut commands: Commands,
    pipelines: Res<SimulationComputePipelines>,
    gpu_buffers: Res<SimulationGpuBuffers>,
    settings: Res<SimulationSettings>,
    config: Res<SimulationConfig>,
    merge_cell: Res<MergeCellCpuState>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    storage: Res<RenderAssets<GpuShaderStorageBuffer>>,
    mut cache: Local<CachedBindGroupParams>,
    mut uniforms: Local<SimulationComputeUniforms>,
) {
    let params = current_params(&settings, &config, &merge_cell);
    if cache.last.as_ref() == Some(&params) {
        return;
    }
    cache.last = Some(params);

    let Some(positions) = storage.get(&gpu_buffers.positions) else {
        return;
    };
    let Some(velocities) = storage.get(&gpu_buffers.velocities) else {
        return;
    };
    let Some(masses) = storage.get(&gpu_buffers.masses) else {
        return;
    };
    let Some(accelerations) = storage.get(&gpu_buffers.accelerations) else {
        return;
    };
    let Some(accelerations_new) = storage.get(&gpu_buffers.accelerations_new) else {
        return;
    };
    let Some(merge_bucket_heads) = storage.get(&gpu_buffers.merge_bucket_heads) else {
        return;
    };
    let Some(merge_aux) = storage.get(&gpu_buffers.merge_aux) else {
        return;
    };
    let Some(merge_owner) = storage.get(&gpu_buffers.merge_owner) else {
        return;
    };
    let Some(merge_scratch) = storage.get(&gpu_buffers.merge_scratch) else {
        return;
    };
    let Some(body_colors) = storage.get(&gpu_buffers.body_colors) else {
        return;
    };

    let (gravity_params, integrate_params, merge_params, colors_params) = params;

    if uniforms.gravity.is_none() {
        uniforms.gravity = Some(UniformBuffer::from(gravity_params));
        uniforms.integrate = Some(UniformBuffer::from(integrate_params));
        uniforms.merge = Some(UniformBuffer::from(merge_params));
        uniforms.colors = Some(UniformBuffer::from(colors_params));
    } else {
        *uniforms.gravity.as_mut().unwrap() = UniformBuffer::from(gravity_params);
        *uniforms.integrate.as_mut().unwrap() = UniformBuffer::from(integrate_params);
        *uniforms.merge.as_mut().unwrap() = UniformBuffer::from(merge_params);
        *uniforms.colors.as_mut().unwrap() = UniformBuffer::from(colors_params);
    }

    uniforms
        .gravity
        .as_mut()
        .unwrap()
        .write_buffer(&render_device, &render_queue);
    uniforms
        .integrate
        .as_mut()
        .unwrap()
        .write_buffer(&render_device, &render_queue);
    uniforms
        .merge
        .as_mut()
        .unwrap()
        .write_buffer(&render_device, &render_queue);
    uniforms
        .colors
        .as_mut()
        .unwrap()
        .write_buffer(&render_device, &render_queue);

    let gravity_uniform = uniforms.gravity.as_ref().unwrap();
    let integrate_uniform = uniforms.integrate.as_ref().unwrap();
    let merge_uniform = uniforms.merge.as_ref().unwrap();
    let colors_uniform = uniforms.colors.as_ref().unwrap();

    let gravity_bind_group = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipelines.gravity_layout),
        &BindGroupEntries::sequential((
            positions.buffer.as_entire_buffer_binding(),
            masses.buffer.as_entire_buffer_binding(),
            accelerations_new.buffer.as_entire_buffer_binding(),
            gravity_uniform,
        )),
    );

    let integrate_bind_group = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipelines.integrate_layout),
        &BindGroupEntries::sequential((
            positions.buffer.as_entire_buffer_binding(),
            velocities.buffer.as_entire_buffer_binding(),
            accelerations.buffer.as_entire_buffer_binding(),
            accelerations_new.buffer.as_entire_buffer_binding(),
            masses.buffer.as_entire_buffer_binding(),
            integrate_uniform,
        )),
    );

    let merge_bind_group = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipelines.merge_layout),
        &BindGroupEntries::sequential((
            positions.buffer.as_entire_buffer_binding(),
            velocities.buffer.as_entire_buffer_binding(),
            masses.buffer.as_entire_buffer_binding(),
            accelerations.buffer.as_entire_buffer_binding(),
            merge_scratch.buffer.as_entire_buffer_binding(),
            merge_bucket_heads.buffer.as_entire_buffer_binding(),
            merge_aux.buffer.as_entire_buffer_binding(),
            merge_owner.buffer.as_entire_buffer_binding(),
            merge_uniform,
        )),
    );

    let colors_bind_group = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipelines.colors_layout),
        &BindGroupEntries::sequential((
            masses.buffer.as_entire_buffer_binding(),
            merge_aux.buffer.as_entire_buffer_binding(),
            body_colors.buffer.as_entire_buffer_binding(),
            colors_uniform,
        )),
    );

    commands.insert_resource(SimulationComputeBindGroups {
        gravity: gravity_bind_group,
        integrate: integrate_bind_group,
        merge: merge_bind_group,
        colors: colors_bind_group,
    });
}
