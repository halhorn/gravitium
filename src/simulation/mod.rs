mod commands;
mod config;
pub mod gpu;
mod merge_cell;
mod playback;
mod profiling;
mod restart;
mod settings;
pub mod shaders;
mod upload;
mod viewport;

pub use commands::{SimulationCommand, SimulationSpawned};
pub use config::SimulationConfig;
pub use gpu::SimulationGpuBuffers;
pub use playback::{PlaybackMode, PlaybackState};
pub use profiling::{
    add_diagnostics_plugins, automated_profiling_active, physics_state_hash, profiling_enabled,
    ProfilingOverlay, SimulationProfilingPlugin,
};
pub use merge_cell::MergeCellCpuState;
pub use restart::restart_simulation;
pub use settings::SimulationSettings;
pub use viewport::{
    fallback_logical_rect, fallback_logical_rect_from_size, logical_rect_to_camera_viewport,
    point_in_simulation_viewport, pick_ray_in_rect, viewport_aspect_from_rect,
    viewport_aspect_from_window, world_to_viewport_in_rect, SimulationViewportRect,
    SimViewportSystems, DESKTOP_PANEL_WIDTH, MOBILE_BREAKPOINT_PX, MOBILE_PANEL_HEIGHT,
};

use bevy::prelude::*;

use gpu::SimulationGpuPlugin;
use playback::tick_sim_time;
use restart::spawn_initial_simulation;
use shaders::register_simulation_shaders;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationRestartSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationStartupSet;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationConfig>()
            .init_resource::<SimulationSettings>()
            .init_resource::<MergeCellCpuState>()
            .init_resource::<PlaybackState>()
            .init_resource::<SimulationViewportRect>()
            .init_resource::<ProfilingOverlay>()
            .add_message::<SimulationSpawned>()
            .add_message::<SimulationCommand>()
            .add_plugins(SimulationGpuPlugin)
            .add_systems(
                Startup,
                (register_simulation_shaders, spawn_initial_simulation)
                    .in_set(SimulationStartupSet),
            )
            .add_systems(Update, tick_sim_time);

        if profiling_enabled() {
            app.add_plugins(SimulationProfilingPlugin);
        }
    }
}
