mod apply_upload;
mod bind_groups;
pub mod buffers;
mod node;
pub mod params;
mod pipelines;

pub use buffers::SimulationGpuBuffers;
pub use node::{SimulationComputeLabel, SimulationComputeNode};

use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_resource::ExtractResourcePlugin,
        render_graph::RenderGraph,
    },
};

use crate::simulation::config::SimulationConfig;
use crate::simulation::merge_cell::MergeCellCpuState;
use crate::simulation::playback::PlaybackState;
use crate::simulation::settings::SimulationSettings;
use crate::simulation::upload::PendingSimulationUpload;

use apply_upload::apply_pending_simulation_upload;
use bind_groups::prepare_simulation_bind_groups;
use pipelines::{init_simulation_compute_pipelines, SimulationComputePipelines};

pub struct SimulationGpuPlugin;

impl Plugin for SimulationGpuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingSimulationUpload>()
            .add_plugins(ExtractResourcePlugin::<SimulationGpuBuffers>::default())
            .add_plugins(ExtractResourcePlugin::<SimulationConfig>::default())
            .add_plugins(ExtractResourcePlugin::<SimulationSettings>::default())
            .add_plugins(ExtractResourcePlugin::<MergeCellCpuState>::default())
            .add_plugins(ExtractResourcePlugin::<PlaybackState>::default())
            .add_plugins(ExtractResourcePlugin::<PendingSimulationUpload>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(RenderStartup, init_simulation_compute_pipelines)
            .add_systems(
                Render,
                (
                    apply_pending_simulation_upload,
                    prepare_simulation_bind_groups,
                )
                    .chain()
                    .in_set(RenderSystems::PrepareBindGroups)
                    .run_if(resource_exists::<SimulationComputePipelines>)
                    .run_if(resource_exists::<SimulationGpuBuffers>)
                    .run_if(resource_exists::<SimulationConfig>)
                    .run_if(resource_exists::<SimulationSettings>)
                    .run_if(resource_exists::<MergeCellCpuState>),
            );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(SimulationComputeLabel, SimulationComputeNode::default());
        render_graph.add_node_edge(
            SimulationComputeLabel,
            bevy::render::graph::CameraDriverLabel,
        );
    }
}
