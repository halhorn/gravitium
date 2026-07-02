use bevy::{
    input::mouse::MouseMotion,
    input::touch::Touches,
    prelude::*,
};
use bevy_egui::EguiPreUpdateSet;
use bevy_panorbit_camera::{
    ActiveCameraData, EguiWantsFocus, PanOrbitCamera, PanOrbitCameraSystemSet, TouchControls,
};

use crate::simulation::SimulationViewportRect;
use crate::view::SimulationCamera;

use super::pivot::zoom_pivot_on_focus_plane;

/// Recompute yaw/pitch/radius for `new_focus` while keeping the camera at `translation`.
pub fn recenter_orbit_focus(
    pan_orbit: &mut PanOrbitCamera,
    translation: Vec3,
    new_focus: Vec3,
) {
    let (yaw, pitch, radius) =
        orbit_params_from_translation(translation, new_focus, pan_orbit.axis);

    pan_orbit.focus = new_focus;
    pan_orbit.target_focus = new_focus;
    pan_orbit.yaw = Some(yaw);
    pan_orbit.pitch = Some(pitch);
    pan_orbit.radius = Some(radius);
    pan_orbit.target_yaw = yaw;
    pan_orbit.target_pitch = pitch;
    pan_orbit.target_radius = radius;
}

/// Default orbit yaw / pitch from the legacy `(0, 80, 120)` camera pose.
pub fn default_orbit_angles(axis: [Vec3; 3]) -> (f32, f32) {
    let (yaw, pitch, _) = orbit_params_from_translation(
        Vec3::new(0.0, 80.0, 120.0),
        Vec3::ZERO,
        axis,
    );
    (yaw, pitch)
}

pub fn camera_transform_from_orbit(
    yaw: f32,
    pitch: f32,
    radius: f32,
    focus: Vec3,
    axis: [Vec3; 3],
) -> Transform {
    let yaw_rot = Quat::from_axis_angle(axis[1], yaw);
    let pitch_rot = Quat::from_axis_angle(axis[0], -pitch);
    let rotation = yaw_rot * pitch_rot;
    let translation = focus + rotation * Vec3::new(0.0, 0.0, radius);
    Transform::from_translation(translation).looking_at(focus, axis[1])
}

pub fn orbit_params_from_translation(
    translation: Vec3,
    focus: Vec3,
    axis: [Vec3; 3],
) -> (f32, f32, f32) {
    let axis = Mat3::from_cols(axis[0], axis[1], axis[2]);
    let offset = axis * (translation - focus);
    let mut radius = offset.length();
    if radius == 0.0 {
        radius = 0.05;
    }
    let yaw = offset.x.atan2(offset.z);
    let pitch = (offset.y / radius).asin();
    (yaw, pitch, radius)
}

pub fn screen_center_orbit_pivot(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    focus: Vec3,
    viewport_center: Vec2,
) -> Option<Vec3> {
    let ray = camera
        .viewport_to_world(camera_transform, viewport_center)
        .ok()?;
    let plane_normal = camera_transform.forward().as_vec3();
    Some(zoom_pivot_on_focus_plane(
        ray.origin,
        *ray.direction,
        focus,
        plane_normal,
    ))
}

fn orbit_pressed(
    pan_orbit: &PanOrbitCamera,
    mouse: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
) -> bool {
    pan_orbit
        .modifier_orbit
        .is_none_or(|modifier| keys.pressed(modifier))
        && mouse.pressed(pan_orbit.button_orbit)
        && pan_orbit
            .modifier_pan
            .is_none_or(|modifier| !keys.pressed(modifier))
}

fn orbit_gesture_active(
    pan_orbit: &PanOrbitCamera,
    mouse: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
    touches: &Touches,
) -> bool {
    if orbit_pressed(pan_orbit, mouse, keys) {
        return true;
    }

    if !pan_orbit.touch_enabled {
        return false;
    }

    let touch_count = touches.iter().count();
    match pan_orbit.touch_controls {
        TouchControls::OneFingerOrbit => touch_count == 1,
        TouchControls::TwoFingerOrbit => touch_count == 2,
    }
}

#[derive(Resource, Default)]
struct OrbitFocusState {
    gesture_active: bool,
    syncing: bool,
}

fn orbit_motion_delta(
    pan_orbit: &PanOrbitCamera,
    mouse: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
    touches: &Touches,
    mouse_motion: &mut MessageReader<MouseMotion>,
) -> Vec2 {
    if orbit_pressed(pan_orbit, mouse, keys) {
        return mouse_motion.read().map(|event| event.delta).sum();
    }

    if !pan_orbit.touch_enabled {
        return Vec2::ZERO;
    }

    let pressed: Vec<_> = touches.iter().collect();
    match pan_orbit.touch_controls {
        TouchControls::OneFingerOrbit if pressed.len() == 1 => pressed[0].delta(),
        TouchControls::TwoFingerOrbit if pressed.len() == 2 => {
            let midpoint = pressed[0].position().midpoint(pressed[1].position());
            let prev_midpoint = pressed[0]
                .previous_position()
                .midpoint(pressed[1].previous_position());
            midpoint - prev_midpoint
        }
        _ => Vec2::ZERO,
    }
}

fn sync_orbit_focus_to_screen_center(
    egui_wants_focus: Res<EguiWantsFocus>,
    viewport_rect: Res<SimulationViewportRect>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut active_cam: ResMut<ActiveCameraData>,
    mut state: ResMut<OrbitFocusState>,
    mut camera: Query<(Entity, &Camera, &GlobalTransform, &mut PanOrbitCamera), With<SimulationCamera>>,
) {
    if egui_wants_focus.prev || egui_wants_focus.curr {
        return;
    }

    let Ok((entity, camera_component, camera_transform, mut pan_orbit)) = camera.single_mut() else {
        return;
    };

    if !pan_orbit.enabled || !pan_orbit.initialized {
        return;
    }

    let gesture_active = orbit_gesture_active(&pan_orbit, &mouse, &keys, &touches);
    if !gesture_active {
        state.gesture_active = false;
        state.syncing = false;
        return;
    }

    if let Some(viewport_size) = camera_component.logical_viewport_size() {
        active_cam.entity = Some(entity);
        active_cam.viewport_size = Some(viewport_size);
        // PanOrbit normalizes orbit delta by window size; use the simulation viewport instead.
        active_cam.window_size = Some(viewport_size);
    }

    if !state.gesture_active {
        state.gesture_active = true;
        state.syncing = false;
    }

    if state.syncing {
        return;
    }

    let motion = orbit_motion_delta(
        &pan_orbit,
        &mouse,
        &keys,
        &touches,
        &mut mouse_motion,
    );
    if motion.length_squared() <= f32::EPSILON {
        return;
    }

    let Some(pivot) = screen_center_orbit_pivot(
        camera_component,
        camera_transform,
        pan_orbit.focus,
        viewport_rect.logical.center(),
    ) else {
        return;
    };

    recenter_orbit_focus(
        &mut pan_orbit,
        camera_transform.translation(),
        pivot,
    );
    state.syncing = true;
}

pub struct OrbitFocusPlugin;

impl Plugin for OrbitFocusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrbitFocusState>().add_systems(
            PostUpdate,
            sync_orbit_focus_to_screen_center
                .after(EguiPreUpdateSet::InitContexts)
                .before(PanOrbitCameraSystemSet),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera_position_from_orbit(
        yaw: f32,
        pitch: f32,
        radius: f32,
        focus: Vec3,
        axis: [Vec3; 3],
    ) -> Vec3 {
        camera_transform_from_orbit(yaw, pitch, radius, focus, axis).translation
    }

    #[test]
    fn recenter_preserves_camera_position() {
        let translation = Vec3::new(0.0, 80.0, 120.0);
        let old_focus = Vec3::ZERO;
        let new_focus = Vec3::new(10.0, 0.0, 0.0);
        let axis = [Vec3::X, Vec3::Y, Vec3::Z];

        let (yaw, pitch, radius) = orbit_params_from_translation(translation, old_focus, axis);
        assert!(camera_position_from_orbit(yaw, pitch, radius, old_focus, axis)
            .abs_diff_eq(translation, 1e-4));

        let (new_yaw, new_pitch, new_radius) =
            orbit_params_from_translation(translation, new_focus, axis);
        assert!(camera_position_from_orbit(new_yaw, new_pitch, new_radius, new_focus, axis)
            .abs_diff_eq(translation, 1e-4));
    }
}
