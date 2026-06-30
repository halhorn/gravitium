use bevy::{
    prelude::*,
    render::{
        diagnostic::RecordDiagnostics,
        render_graph::{self, RenderLabel},
        render_resource::*,
    },
    shader::PipelineCacheError,
};

use crate::model::constants::{MERGE_BUCKET_COUNT, MERGE_ITERATIONS_PER_FRAME, WORKGROUP_SIZE};
use crate::model::MergeCellSizePolicy;
use crate::simulation::playback::PlaybackState;
use crate::simulation::settings::SimulationSettings;

use super::bind_groups::SimulationComputeBindGroups;
use super::pipelines::SimulationComputePipelines;

fn dispatch_workgroups(active_count: u32) -> u32 {
    active_count.div_ceil(WORKGROUP_SIZE)
}

fn run_compute_pass(
    render_context: &mut bevy::render::renderer::RenderContext,
    diagnostics: &impl RecordDiagnostics,
    pass_name: &'static str,
    record: impl FnOnce(&mut ComputePass<'_>),
) {
    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some(pass_name),
            ..default()
        });
    let span = diagnostics.pass_span(&mut pass, pass_name);
    record(&mut pass);
    span.end(&mut pass);
}

fn run_merge_find_and_apply(
    render_context: &mut bevy::render::renderer::RenderContext,
    diagnostics: &impl RecordDiagnostics,
    bind_groups: &SimulationComputeBindGroups,
    find_owner: &ComputePipeline,
    apply_merge: &ComputePipeline,
    workgroups: u32,
) {
    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("sim/merge_find_and_apply"),
            ..default()
        });

    let find_span = diagnostics.pass_span(&mut pass, "sim/merge_find_owner");
    pass.set_bind_group(0, &bind_groups.merge, &[]);
    pass.set_pipeline(find_owner);
    pass.dispatch_workgroups(workgroups, 1, 1);
    find_span.end(&mut pass);

    let apply_span = diagnostics.pass_span(&mut pass, "sim/merge_apply");
    pass.set_bind_group(0, &bind_groups.merge, &[]);
    pass.set_pipeline(apply_merge);
    pass.dispatch_workgroups(workgroups, 1, 1);
    apply_span.end(&mut pass);
}

fn run_merge_finalize_cell_size(
    render_context: &mut bevy::render::renderer::RenderContext,
    diagnostics: &impl RecordDiagnostics,
    bind_groups: &SimulationComputeBindGroups,
    pipeline: &ComputePipeline,
) {
    run_compute_pass(
        render_context,
        diagnostics,
        "sim/merge_finalize_cell_size",
        |pass| {
            pass.set_bind_group(0, &bind_groups.merge, &[]);
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(1, 1, 1);
        },
    );
}

fn run_merge_iteration(
    render_context: &mut bevy::render::renderer::RenderContext,
    diagnostics: &impl RecordDiagnostics,
    bind_groups: &SimulationComputeBindGroups,
    pipelines: &SimulationComputePipelines,
    cache: &PipelineCache,
    workgroups: u32,
    bucket_workgroups: u32,
) {
    let merge_clear_buckets = cache
        .get_compute_pipeline(pipelines.merge_clear_buckets)
        .unwrap();
    let merge_init_owner = cache.get_compute_pipeline(pipelines.merge_init_owner).unwrap();
    let merge_build_grid = cache.get_compute_pipeline(pipelines.merge_build_grid).unwrap();
    let merge_find_owner = cache.get_compute_pipeline(pipelines.merge_find_owner).unwrap();
    let merge_apply = cache.get_compute_pipeline(pipelines.merge_apply).unwrap();

    run_compute_pass(
        render_context,
        diagnostics,
        "sim/merge_clear_buckets",
        |pass| {
            pass.set_bind_group(0, &bind_groups.merge, &[]);
            pass.set_pipeline(merge_clear_buckets);
            pass.dispatch_workgroups(bucket_workgroups, 1, 1);
        },
    );

    run_compute_pass(
        render_context,
        diagnostics,
        "sim/merge_init_owner",
        |pass| {
            pass.set_bind_group(0, &bind_groups.merge, &[]);
            pass.set_pipeline(merge_init_owner);
            pass.dispatch_workgroups(workgroups, 1, 1);
        },
    );

    run_compute_pass(
        render_context,
        diagnostics,
        "sim/merge_build_grid",
        |pass| {
            pass.set_bind_group(0, &bind_groups.merge, &[]);
            pass.set_pipeline(merge_build_grid);
            pass.dispatch_workgroups(workgroups, 1, 1);
        },
    );

    run_merge_find_and_apply(
        render_context,
        diagnostics,
        bind_groups,
        merge_find_owner,
        merge_apply,
        workgroups,
    );
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SimulationComputeLabel;

#[derive(Default)]
pub struct SimulationComputeNode {
    ready: bool,
}

impl render_graph::Node for SimulationComputeNode {
    fn update(&mut self, world: &mut World) {
        if self.ready {
            return;
        }
        let pipelines = world.resource::<SimulationComputePipelines>();
        let cache = world.resource::<PipelineCache>();
        for id in [
            pipelines.gravity,
            pipelines.position_step,
            pipelines.velocity_step,
            pipelines.merge_prepare,
            pipelines.merge_finalize_cell_size,
            pipelines.merge_clear_buckets,
            pipelines.merge_init_owner,
            pipelines.merge_build_grid,
            pipelines.merge_find_owner,
            pipelines.merge_apply,
            pipelines.colors,
        ] {
            match cache.get_compute_pipeline_state(id) {
                CachedPipelineState::Ok(_) => {}
                CachedPipelineState::Err(PipelineCacheError::ShaderNotLoaded(_)) => return,
                CachedPipelineState::Err(err) => {
                    bevy::log::error!("simulation compute pipeline: {err}");
                    return;
                }
                _ => return,
            }
        }
        self.ready = true;
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if !self.ready {
            return Ok(());
        }
        let Some(bind_groups) = world.get_resource::<SimulationComputeBindGroups>() else {
            return Ok(());
        };

        let pipelines = world.resource::<SimulationComputePipelines>();
        let cache = world.resource::<PipelineCache>();
        let settings = world.resource::<SimulationSettings>();
        let workgroups = dispatch_workgroups(settings.active_count());
        let bucket_workgroups = (MERGE_BUCKET_COUNT as u32).div_ceil(256);
        let diagnostics = render_context.diagnostic_recorder();
        let playback = world.resource::<PlaybackState>();
        let adaptive_cell_size =
            settings.physics.merge_cell_policy == MergeCellSizePolicy::AdaptivePerPrepare;

        if playback.is_running() {
            let gravity = cache.get_compute_pipeline(pipelines.gravity).unwrap();
            let position_step = cache.get_compute_pipeline(pipelines.position_step).unwrap();
            let velocity_step = cache.get_compute_pipeline(pipelines.velocity_step).unwrap();
            let merge_prepare = cache.get_compute_pipeline(pipelines.merge_prepare).unwrap();
            let merge_finalize_cell_size = cache
                .get_compute_pipeline(pipelines.merge_finalize_cell_size)
                .unwrap();

            run_compute_pass(
                render_context,
                &diagnostics,
                "sim/position_step",
                |pass| {
                    pass.set_bind_group(0, &bind_groups.integrate, &[]);
                    pass.set_pipeline(position_step);
                    pass.dispatch_workgroups(workgroups, 1, 1);
                },
            );

            run_compute_pass(
                render_context,
                &diagnostics,
                "sim/gravity",
                |pass| {
                    pass.set_bind_group(0, &bind_groups.gravity, &[]);
                    pass.set_pipeline(gravity);
                    pass.dispatch_workgroups(workgroups, 1, 1);
                },
            );

            run_compute_pass(
                render_context,
                &diagnostics,
                "sim/velocity_step",
                |pass| {
                    pass.set_bind_group(0, &bind_groups.integrate, &[]);
                    pass.set_pipeline(velocity_step);
                    pass.dispatch_workgroups(workgroups, 1, 1);
                },
            );

            for _ in 0..MERGE_ITERATIONS_PER_FRAME {
                run_compute_pass(
                    render_context,
                    &diagnostics,
                    "sim/merge_prepare",
                    |pass| {
                        pass.set_bind_group(0, &bind_groups.merge, &[]);
                        pass.set_pipeline(merge_prepare);
                        pass.dispatch_workgroups(workgroups, 1, 1);
                    },
                );

                if adaptive_cell_size {
                    run_merge_finalize_cell_size(
                        render_context,
                        &diagnostics,
                        bind_groups,
                        merge_finalize_cell_size,
                    );
                }

                run_merge_iteration(
                    render_context,
                    &diagnostics,
                    bind_groups,
                    pipelines,
                    cache,
                    workgroups,
                    bucket_workgroups,
                );
            }
        }

        // O(n) color pass — runs while paused so body colors stay in sync with mass/flash state.
        let colors = cache.get_compute_pipeline(pipelines.colors).unwrap();
        run_compute_pass(
            render_context,
            &diagnostics,
            "sim/colors",
            |pass| {
                pass.set_bind_group(0, &bind_groups.colors, &[]);
                pass.set_pipeline(colors);
                pass.dispatch_workgroups(workgroups, 1, 1);
            },
        );

        Ok(())
    }
}
