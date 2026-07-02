use bevy::prelude::*;
use bevy::camera::Viewport;

/// Viewport width below which the control panel moves to the bottom (phone / narrow window).
pub const MOBILE_BREAKPOINT_PX: f32 = 768.0;
pub const MOBILE_PANEL_HEIGHT: f32 = 300.0;
pub const DESKTOP_PANEL_WIDTH: f32 = 300.0;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimViewportSystems {
    /// egui control panel layout (runs first).
    Layout,
    /// 3D camera viewport after panel rect is known (runs second).
    CameraViewport,
}

/// Logical screen rect (top-left origin) reserved for the 3D simulation view.
#[derive(Resource, Clone, Copy, Debug)]
pub struct SimulationViewportRect {
    pub logical: Rect,
}

impl Default for SimulationViewportRect {
    fn default() -> Self {
        Self {
            logical: Rect::from_corners(Vec2::ZERO, Vec2::ONE),
        }
    }
}

pub fn fallback_logical_rect(window: &Window) -> Rect {
    fallback_logical_rect_from_size(window.width(), window.height())
}

pub fn fallback_logical_rect_from_size(window_width: f32, window_height: f32) -> Rect {
    if window_width < MOBILE_BREAKPOINT_PX {
        Rect {
            min: Vec2::ZERO,
            max: Vec2::new(window_width, (window_height - MOBILE_PANEL_HEIGHT).max(1.0)),
        }
    } else {
        Rect {
            min: Vec2::new(DESKTOP_PANEL_WIDTH, 0.0),
            max: Vec2::new(window_width, window_height),
        }
    }
}

pub fn viewport_aspect_from_window(window: &Window) -> f32 {
    let rect = fallback_logical_rect(window);
    (rect.width() / rect.height().max(1e-6)).max(1e-6)
}

pub fn viewport_aspect_from_rect(rect: Rect) -> f32 {
    (rect.width() / rect.height().max(1e-6)).max(1e-6)
}

/// WebGPU / Metal `max_texture_dimension_2d` on many Apple GPUs.
const MAX_VIEWPORT_PHYSICAL: u32 = 8192;

/// Whether `point` (window logical coords) lies in the 3D simulation viewport.
pub fn point_in_simulation_viewport(
    point: Vec2,
    viewport_rect: &SimulationViewportRect,
    _camera: &Camera,
) -> bool {
    viewport_rect.logical.contains(point)
}

/// Project a world position to window logical coordinates using the egui simulation rect.
///
/// Prefer this over [`Camera::world_to_viewport`] when the camera viewport is driven by
/// [`SimulationViewportRect`]; Bevy's built-in helper can disagree on the logical rect.
pub fn world_to_viewport_in_rect(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    world: Vec3,
    viewport: Rect,
) -> Option<Vec2> {
    let ndc = camera.world_to_ndc(camera_transform, world)?;
    if !(0.0..=1.0).contains(&ndc.z) {
        return None;
    }
    Some(ndc_to_viewport_point(ndc.truncate(), viewport))
}

fn ndc_to_viewport_point(ndc_xy: Vec2, viewport: Rect) -> Vec2 {
    let mut ndc = ndc_xy;
    ndc.y = -ndc.y;
    (ndc + Vec2::ONE) / 2.0 * viewport.size() + viewport.min
}

fn viewport_point_to_ndc(cursor: Vec2, viewport: Rect) -> Vec2 {
    let rel = (cursor - viewport.min) / viewport.size();
    let mut ndc = rel * 2.0 - Vec2::ONE;
    ndc.y = -ndc.y;
    ndc
}

/// World-space ray through `cursor` using the same viewport mapping as [`world_to_viewport_in_rect`].
pub fn pick_ray_in_rect(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor: Vec2,
    viewport: Rect,
) -> Option<Ray3d> {
    let ndc_xy = viewport_point_to_ndc(cursor, viewport);
    let origin = camera.ndc_to_world(camera_transform, ndc_xy.extend(1.0))?;
    let far = camera.ndc_to_world(camera_transform, ndc_xy.extend(f32::EPSILON))?;
    let direction = Dir3::new(far - origin).ok()?;
    Some(Ray3d { origin, direction })
}

pub fn logical_rect_to_camera_viewport(rect: Rect, window: &Window) -> Viewport {
    let scale = window.resolution.scale_factor();

    let mut phys_w = (rect.width().max(1.0) * scale).round() as u32;
    let mut phys_h = (rect.height().max(1.0) * scale).round() as u32;
    phys_w = phys_w.min(window.physical_width()).min(MAX_VIEWPORT_PHYSICAL);
    phys_h = phys_h.min(window.physical_height()).min(MAX_VIEWPORT_PHYSICAL);

    Viewport {
        physical_position: UVec2::new(
            (rect.min.x * scale).round() as u32,
            (rect.min.y * scale).round() as u32,
        ),
        physical_size: UVec2::new(phys_w.max(1), phys_h.max(1)),
        depth: 0.0..1.0,
    }
}
