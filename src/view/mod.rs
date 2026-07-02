mod bodies;
mod camera;
mod selection;
mod sim_viewport;

pub use bodies::{setup_bodies_render, BodiesMesh, BodiesRenderPlugin};
pub use selection::SimulationCpuSnapshot;
pub use camera::{
    default_simulation_camera_transform, default_simulation_pan_orbit,
    fallback_viewport_aspect, simulation_camera_for_outer_radius,
};
pub use sim_viewport::{SimulationCamera, SIMULATION_RENDER_LAYER, UI_RENDER_LAYER};

use bevy::prelude::*;

use camera::{CameraControlsPlugin, OrbitFocusPlugin};
use selection::SelectionPlugin;
use sim_viewport::SimulationViewportPlugin;

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BodiesRenderPlugin,
            SelectionPlugin,
            SimulationViewportPlugin,
            CameraControlsPlugin,
            OrbitFocusPlugin,
        ));
    }
}
