use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::model::constants::DISK_R_OUTER;

const DEFAULT_CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 80.0, 120.0);
const DEFAULT_ORBIT_RADIUS: f32 = 144.22205;

/// Orbit distance so framing matches `outer_radius`, calibrated at the default 60 AU disk.
pub fn orbit_radius_for_outer_radius(outer_radius: f32) -> f32 {
    DEFAULT_ORBIT_RADIUS * (outer_radius / DISK_R_OUTER)
}

pub fn simulation_camera_transform(outer_radius: f32) -> Transform {
    let scale = orbit_radius_for_outer_radius(outer_radius) / DEFAULT_ORBIT_RADIUS;
    Transform::from_translation(DEFAULT_CAMERA_OFFSET * scale).looking_at(Vec3::ZERO, Vec3::Y)
}

pub fn simulation_pan_orbit(outer_radius: f32) -> PanOrbitCamera {
    let radius = orbit_radius_for_outer_radius(outer_radius);
    PanOrbitCamera {
        zoom_sensitivity: 0.0,
        radius: Some(radius),
        target_radius: radius,
        ..default()
    }
}

pub fn default_simulation_camera_transform() -> Transform {
    simulation_camera_transform(DISK_R_OUTER)
}

pub fn default_simulation_pan_orbit() -> PanOrbitCamera {
    simulation_pan_orbit(DISK_R_OUTER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbit_radius_scales_linearly_with_outer_radius() {
        let r60 = orbit_radius_for_outer_radius(60.0);
        assert!((r60 - DEFAULT_ORBIT_RADIUS).abs() < 1e-3);
        assert!((orbit_radius_for_outer_radius(20.0) - r60 * (20.0 / 60.0)).abs() < 1e-3);
        assert!((orbit_radius_for_outer_radius(150.0) - r60 * (150.0 / 60.0)).abs() < 1e-3);
    }
}
