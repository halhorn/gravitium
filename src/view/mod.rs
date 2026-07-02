mod bodies;
mod camera;
mod selection;
mod sim_viewport;

pub use bodies::{BodiesMesh, BodiesRenderPlugin, setup_bodies_render};
pub use camera::{
    default_simulation_camera_transform, default_simulation_pan_orbit, simulation_camera_transform,
    simulation_pan_orbit,
};
pub use selection::SimulationCpuSnapshot;
pub use sim_viewport::{SIMULATION_RENDER_LAYER, SimulationCamera, UI_RENDER_LAYER};

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
