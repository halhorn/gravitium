use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::simulation::{
    SimulationCommand, SimulationSettings, SimulationViewportRect, viewport_aspect_from_rect,
    viewport_aspect_from_window,
};
use crate::view::SimulationCamera;

use super::defaults::{
    simulation_camera_for_outer_radius, vertical_fov_from_projection,
};

fn simulation_viewport_aspect(
    viewport_rect: &SimulationViewportRect,
    window: &Window,
) -> f32 {
    let rect = viewport_rect.logical;
    if rect.width() > 1.0 && rect.height() > 1.0 {
        viewport_aspect_from_rect(rect)
    } else {
        viewport_aspect_from_window(window)
    }
}

pub fn reset_simulation_camera_on_restart(
    mut commands: MessageReader<SimulationCommand>,
    settings: Res<SimulationSettings>,
    viewport_rect: Res<SimulationViewportRect>,
    windows: Query<&Window>,
    mut camera: Query<
        (&mut Transform, &mut PanOrbitCamera, &Projection),
        With<SimulationCamera>,
    >,
) {
    if !commands
        .read()
        .any(|command| matches!(command, SimulationCommand::Restart))
    {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((mut transform, mut pan_orbit, projection)) = camera.single_mut() else {
        return;
    };

    let aspect = simulation_viewport_aspect(&viewport_rect, window);
    let vertical_fov = vertical_fov_from_projection(projection);
    let (new_transform, new_pan_orbit) = simulation_camera_for_outer_radius(
        settings.initial.disk_r_max,
        vertical_fov,
        aspect,
    );

    *transform = new_transform;
    *pan_orbit = new_pan_orbit;
}
