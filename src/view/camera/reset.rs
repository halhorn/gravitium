use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::simulation::{SimulationCommand, SimulationSettings};
use crate::view::SimulationCamera;

use super::defaults::{simulation_camera_transform, simulation_pan_orbit};

/// Apply zoom from `disk_r_max` after URL hydration (Startup order is not guaranteed).
pub fn sync_initial_camera_to_outer_radius(
    settings: Res<SimulationSettings>,
    mut camera: Query<(&mut Transform, &mut PanOrbitCamera), With<SimulationCamera>>,
) {
    let Ok((mut transform, mut pan_orbit)) = camera.single_mut() else {
        return;
    };

    let outer = settings.initial.disk_r_max;
    *transform = simulation_camera_transform(outer);
    *pan_orbit = simulation_pan_orbit(outer);
}

pub fn reset_simulation_camera_on_restart(
    mut commands: MessageReader<SimulationCommand>,
    settings: Res<SimulationSettings>,
    mut camera: Query<(&mut Transform, &mut PanOrbitCamera), With<SimulationCamera>>,
) {
    if !commands
        .read()
        .any(|command| matches!(command, SimulationCommand::Restart))
    {
        return;
    }

    let Ok((mut transform, mut pan_orbit)) = camera.single_mut() else {
        return;
    };

    let outer = settings.initial.disk_r_max;
    *transform = simulation_camera_transform(outer);
    *pan_orbit = simulation_pan_orbit(outer);
}
