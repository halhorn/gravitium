//! Web 向け Bevy アプリの組み立て。

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::camera::ClearColorConfig;
use bevy::render::settings::{RenderCreation, WgpuSettings, WgpuSettingsPriority};
use bevy::render::RenderPlugin;
use bevy_egui::{EguiContext, PrimaryEguiContext};
use bevy_panorbit_camera::PanOrbitCameraPlugin;

use crate::platform;
use crate::simulation::{
    add_diagnostics_plugins, automated_profiling_active, profiling_enabled, SimulationPlugin,
    SimulationSettings, viewport_aspect_from_window,
};
use crate::url::{UrlNavigation, UrlSyncPlugin};
use crate::view::{
    fallback_viewport_aspect, setup_bodies_render, simulation_camera_for_outer_radius, BodiesMesh,
    SimulationCamera, ViewPlugin, SIMULATION_RENDER_LAYER, UI_RENDER_LAYER,
};
use crate::ui::ControlUiPlugin;

/// ネイティブ・WASM 共通の `App` を組み立てて実行する。
pub fn run() {
    let mut app = App::new();
    let bench_window = automated_profiling_active();
    app.add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                // Safari / iOS WebGPU は Functionality 優先だとパイプラインが落ちることがある。
                priority: WgpuSettingsPriority::Compatibility,
                ..default()
            }),
            ..default()
        }).set(WindowPlugin {
            primary_window: Some(Window {
                title: "Gravitium — Gravity Simulator".to_string(),
                canvas: Some("#gravitium-canvas".into()),
                fit_canvas_to_parent: !bench_window,
                prevent_default_event_handling: true,
                #[cfg(not(target_arch = "wasm32"))]
                resolution: if bench_window {
                    bevy::window::WindowResolution::new(1280, 720)
                        .with_scale_factor_override(1.0)
                } else {
                    bevy::window::WindowResolution::default()
                },
                resizable: !bench_window,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(UrlNavigation(platform::url_navigation_arc()));

    if profiling_enabled() {
        add_diagnostics_plugins(&mut app);
    }

    app.add_plugins((
            UrlSyncPlugin,
            PanOrbitCameraPlugin,
            SimulationPlugin,
            ViewPlugin,
            ControlUiPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(PostStartup, setup_bodies_render)
        .add_systems(Startup, setup_camera)
        .add_systems(Update, hide_loading_when_ready)
        .run();
}

fn setup_camera(mut commands: Commands, settings: Res<SimulationSettings>, windows: Query<&Window>) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(UI_RENDER_LAYER),
        EguiContext::default(),
        PrimaryEguiContext,
    ));

    let aspect = windows
        .single()
        .map(viewport_aspect_from_window)
        .unwrap_or_else(|_| {
            use crate::simulation::DESKTOP_PANEL_WIDTH;
            fallback_viewport_aspect(1280.0 - DESKTOP_PANEL_WIDTH, 720.0)
        });
    let (transform, pan_orbit) = simulation_camera_for_outer_radius(
        settings.initial.disk_r_max,
        std::f32::consts::FRAC_PI_4,
        aspect,
    );

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        SimulationCamera,
        RenderLayers::layer(SIMULATION_RENDER_LAYER),
        transform,
        pan_orbit,
    ));
}

/// シミュレーションの描画エンティティが揃ってからローディング UI を消す。
fn hide_loading_when_ready(bodies: Query<(), With<BodiesMesh>>, mut done: Local<bool>) {
    if *done || bodies.is_empty() {
        return;
    }
    *done = true;
    #[cfg(target_arch = "wasm32")]
    hide_web_loading_overlay();
}

/// `index.html` のローディングオーバーレイを非表示にする。
#[cfg(target_arch = "wasm32")]
fn hide_web_loading_overlay() {
    let Some(win) = web_sys::window() else {
        return;
    };
    let Some(doc) = win.document() else {
        return;
    };
    let Some(el) = doc.get_element_by_id("gravitium-loading") else {
        return;
    };
    let _ = el.set_attribute("hidden", "");
    let _ = el.set_attribute("aria-busy", "false");
}
