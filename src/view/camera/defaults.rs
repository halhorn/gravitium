use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::model::constants::DISK_R_OUTER;
use crate::simulation::{
    fallback_logical_rect_from_size, viewport_aspect_from_rect, DESKTOP_PANEL_WIDTH,
};

use super::orbit_focus::{camera_transform_from_orbit, default_orbit_angles};

pub const DEFAULT_CAMERA_FOCUS: Vec3 = Vec3::ZERO;
pub const DEFAULT_ORBIT_AXIS: [Vec3; 3] = [Vec3::X, Vec3::Y, Vec3::Z];

/// Horizontal field of view in radians from vertical FOV and viewport aspect.
pub fn horizontal_fov_radians(vertical_fov: f32, aspect: f32) -> f32 {
    let aspect = aspect.max(1e-6);
    2.0 * ((vertical_fov * 0.5).tan() * aspect).atan()
}

/// Pan-orbit radius so the horizontal span at the focus plane matches `horizontal_extent`.
pub fn orbit_radius_for_horizontal_extent(
    horizontal_extent: f32,
    vertical_fov: f32,
    aspect: f32,
) -> f32 {
    let half_tan = (horizontal_fov_radians(vertical_fov, aspect) * 0.5)
        .tan()
        .max(1e-6);
    (horizontal_extent * 0.5 / half_tan).max(1e-3)
}

pub fn vertical_fov_from_projection(projection: &Projection) -> f32 {
    match projection {
        Projection::Perspective(perspective) => perspective.fov,
        _ => FRAC_PI_4,
    }
}

/// Fallback aspect when the simulation viewport has not been laid out yet.
pub fn fallback_viewport_aspect(window_width: f32, window_height: f32) -> f32 {
    viewport_aspect_from_rect(fallback_logical_rect_from_size(
        window_width.max(1.0),
        window_height.max(1.0),
    ))
}

pub fn simulation_camera_for_outer_radius(
    outer_radius: f32,
    vertical_fov: f32,
    aspect: f32,
) -> (Transform, PanOrbitCamera) {
    let (yaw, pitch) = default_orbit_angles(DEFAULT_ORBIT_AXIS);
    let radius = orbit_radius_for_horizontal_extent(outer_radius, vertical_fov, aspect);
    let transform =
        camera_transform_from_orbit(yaw, pitch, radius, DEFAULT_CAMERA_FOCUS, DEFAULT_ORBIT_AXIS);

    let mut pan_orbit = PanOrbitCamera {
        zoom_sensitivity: 0.0,
        ..default()
    };
    pan_orbit.focus = DEFAULT_CAMERA_FOCUS;
    pan_orbit.target_focus = DEFAULT_CAMERA_FOCUS;
    pan_orbit.yaw = Some(yaw);
    pan_orbit.pitch = Some(pitch);
    pan_orbit.radius = Some(radius);
    pan_orbit.target_yaw = yaw;
    pan_orbit.target_pitch = pitch;
    pan_orbit.target_radius = radius;

    (transform, pan_orbit)
}

pub fn default_simulation_camera_transform() -> Transform {
    simulation_camera_for_outer_radius(
        DISK_R_OUTER,
        FRAC_PI_4,
        fallback_viewport_aspect(1280.0 - DESKTOP_PANEL_WIDTH, 720.0),
    )
    .0
}

pub fn default_simulation_pan_orbit() -> PanOrbitCamera {
    simulation_camera_for_outer_radius(
        DISK_R_OUTER,
        FRAC_PI_4,
        fallback_viewport_aspect(1280.0 - DESKTOP_PANEL_WIDTH, 720.0),
    )
    .1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbit_radius_matches_requested_horizontal_extent() {
        let aspect = 980.0 / 720.0;
        let vertical_fov = FRAC_PI_4;
        let outer_radius = 60.0;
        let radius = orbit_radius_for_horizontal_extent(outer_radius, vertical_fov, aspect);
        let span = 2.0 * radius * (horizontal_fov_radians(vertical_fov, aspect) * 0.5).tan();
        assert!((span - outer_radius).abs() < 1e-3);
    }

    #[test]
    fn larger_outer_radius_yields_larger_orbit_radius() {
        let aspect = 1.5;
        let vertical_fov = FRAC_PI_4;
        let small = orbit_radius_for_horizontal_extent(30.0, vertical_fov, aspect);
        let large = orbit_radius_for_horizontal_extent(120.0, vertical_fov, aspect);
        assert!(large > small);
    }
}
