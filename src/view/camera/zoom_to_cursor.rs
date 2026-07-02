use bevy::{
    input::gestures::PinchGesture,
    input::mouse::{MouseScrollUnit, MouseWheel},
    input::touch::Touches,
    prelude::*,
};
use bevy_egui::{EguiPreUpdateSet, EguiPrimaryContextPass};
use bevy_panorbit_camera::{EguiWantsFocus, PanOrbitCamera, PanOrbitCameraSystemSet};

use crate::simulation::{
    point_in_simulation_viewport, restart_simulation, SimulationRestartSet,
    SimulationViewportRect,
};
use crate::view::SimulationCamera;

use super::pivot::zoom_pivot_on_focus_plane;
use super::reset::reset_simulation_camera_on_restart;

/// Zoom delta applied to `PanOrbitCamera` targets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomTargetUpdate {
    pub target_focus: Vec3,
    pub target_radius: f32,
}

/// Converts scroll / pinch deltas into `target_focus` and `target_radius` updates
/// that dolly toward `pivot`, matching panorbit 0.34 zoom sensitivity.
pub fn apply_zoom_to_pivot(
    focus: Vec3,
    radius: f32,
    pivot: Vec3,
    scroll_line: f32,
    scroll_pixel: f32,
) -> ZoomTargetUpdate {
    let radius = radius.max(1e-6);
    let delta_r = -(scroll_line + scroll_pixel) * radius * 0.2;
    let alpha = -delta_r / radius;

    ZoomTargetUpdate {
        target_focus: focus + (pivot - focus) * alpha,
        target_radius: radius + delta_r,
    }
}

#[derive(Resource, Default)]
struct TouchPinchState {
    prev_distance: Option<f32>,
}

fn zoom_to_cursor_system(
    mut scroll_events: MessageReader<MouseWheel>,
    mut pinch_events: MessageReader<PinchGesture>,
    touches: Res<Touches>,
    mut touch_pinch: ResMut<TouchPinchState>,
    egui_wants_focus: Res<EguiWantsFocus>,
    viewport_rect: Res<SimulationViewportRect>,
    windows: Query<&Window>,
    mut camera: Query<
        (
            &Camera,
            &GlobalTransform,
            &mut PanOrbitCamera,
        ),
        With<SimulationCamera>,
    >,
) {
    if egui_wants_focus.prev || egui_wants_focus.curr {
        return;
    }

    let scroll_vec: Vec<MouseWheel> = scroll_events.read().cloned().collect();
    let pinch_sum: f32 = pinch_events.read().map(|event| event.0).sum();

    let touch_pinch_input = touch_pinch_delta(&touches, &mut touch_pinch);

    if scroll_vec.is_empty() && pinch_sum.abs() <= f32::EPSILON && touch_pinch_input.is_none() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera_component, camera_transform, mut pan_orbit)) = camera.single_mut() else {
        return;
    };

    if !pan_orbit.enabled || !pan_orbit.initialized {
        return;
    }

    let zoom_direction = if pan_orbit.reversed_zoom { -1.0 } else { 1.0 };
    let (scroll_line, scroll_pixel) = scroll_delta_from_wheel(&scroll_vec, zoom_direction);

    let mut total_pixel = scroll_pixel
        + pinch_sum * 10.0 * pan_orbit.trackpad_sensitivity * zoom_direction;

    let (screen_pos, touch_pixel) = if pan_orbit.touch_enabled {
        match touch_pinch_input {
            Some((midpoint, pixel)) => (Some(midpoint), pixel),
            None => (window.cursor_position(), 0.0),
        }
    } else {
        (window.cursor_position(), 0.0)
    };
    total_pixel += touch_pixel * zoom_direction;

    if (scroll_line + total_pixel).abs() <= f32::EPSILON {
        return;
    }

    let Some(screen_pos) = screen_pos else {
        return;
    };

    if !point_in_simulation_viewport(screen_pos, &viewport_rect, camera_component) {
        return;
    }

    let Ok(ray) = camera_component.viewport_to_world(camera_transform, screen_pos) else {
        return;
    };

    let plane_normal = camera_transform.forward().as_vec3();
    let pivot = zoom_pivot_on_focus_plane(ray.origin, *ray.direction, pan_orbit.focus, plane_normal);

    let radius = pan_orbit.radius.unwrap_or(pan_orbit.target_radius);
    let update = apply_zoom_to_pivot(
        pan_orbit.focus,
        radius,
        pivot,
        scroll_line,
        total_pixel,
    );

    pan_orbit.target_focus = update.target_focus;
    pan_orbit.target_radius = update.target_radius;
}

fn scroll_delta_from_wheel(events: &[MouseWheel], zoom_direction: f32) -> (f32, f32) {
    events.iter().fold((0.0, 0.0), |(line, pixel), event| match event.unit {
        MouseScrollUnit::Line => (line + event.y * zoom_direction, pixel),
        MouseScrollUnit::Pixel => (line, pixel + event.y * 0.005 * zoom_direction),
    })
}

fn touch_pinch_delta(touches: &Touches, state: &mut TouchPinchState) -> Option<(Vec2, f32)> {
    let pressed: Vec<_> = touches.iter().collect();
    if pressed.len() != 2 {
        state.prev_distance = None;
        return None;
    }

    let pos1 = pressed[0].position();
    let pos2 = pressed[1].position();
    let midpoint = pos1.midpoint(pos2);
    let distance = pos1.distance(pos2);

    let pinch_pixel = state
        .prev_distance
        .map(|prev| (distance - prev) * 0.015)
        .unwrap_or(0.0);
    state.prev_distance = Some(distance);

    (pinch_pixel.abs() > f32::EPSILON).then_some((midpoint, pinch_pixel))
}

pub struct CameraControlsPlugin;

impl Plugin for CameraControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchPinchState>()
            .add_systems(
                PostUpdate,
                zoom_to_cursor_system
                    .after(EguiPreUpdateSet::InitContexts)
                    .before(PanOrbitCameraSystemSet),
            )
            .add_systems(
                EguiPrimaryContextPass,
                reset_simulation_camera_on_restart
                    .in_set(SimulationRestartSet)
                    .after(restart_simulation),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_shifts_focus_toward_pivot() {
        let focus = Vec3::ZERO;
        let pivot = Vec3::new(10.0, 0.0, 0.0);
        let update = apply_zoom_to_pivot(focus, 100.0, pivot, 1.0, 0.0);

        assert!(update.target_focus.x > 0.0);
        assert!(update.target_radius < 100.0);
    }

    #[test]
    fn zoom_at_focus_only_changes_radius() {
        let focus = Vec3::new(3.0, 4.0, 5.0);
        let update = apply_zoom_to_pivot(focus, 50.0, focus, 0.5, 0.0);

        assert!(update.target_focus.abs_diff_eq(focus, 1e-5));
        assert!(update.target_radius < 50.0);
    }

    #[test]
    fn scroll_line_and_pixel_combine() {
        let update = apply_zoom_to_pivot(Vec3::ZERO, 100.0, Vec3::X, 1.0, 0.5);
        let line_only = apply_zoom_to_pivot(Vec3::ZERO, 100.0, Vec3::X, 1.0, 0.0);
        let pixel_only = apply_zoom_to_pivot(Vec3::ZERO, 100.0, Vec3::X, 0.0, 0.5);

        assert_ne!(update.target_radius, line_only.target_radius);
        assert_ne!(update.target_radius, pixel_only.target_radius);
    }
}
