//! Simulation GPU/CPU pass timing via Bevy render diagnostics.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bevy::{
    diagnostic::{DiagnosticPath, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::diagnostic::RenderDiagnosticsPlugin,
};
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::model::constants::MIN_MASS;
use crate::simulation::settings::SimulationSettings;
use crate::simulation::SimViewportSystems;
use crate::view::SimulationCpuSnapshot;

/// Diagnostic path prefix for simulation compute passes (`render/sim/...`).
pub const SIM_PASS_PREFIX: &str = "sim";

/// `(diagnostic suffix, display label)` for each instrumented compute pass.
pub const SIM_PASSES: &[(&str, &str)] = &[
    ("position_step", "Position step"),
    ("gravity", "Gravity"),
    ("velocity_step", "Velocity step"),
    ("merge_prepare", "Merge prepare"),
    ("merge_finalize_cell_size", "Merge finalize cell size"),
    ("merge_clear_buckets", "Merge clear buckets"),
    ("merge_init_owner", "Merge init owner"),
    ("merge_build_grid", "Merge build grid"),
    ("merge_find_owner", "Merge find owner"),
    ("merge_apply", "Merge apply"),
    ("colors", "Colors"),
];

const MERGE_PASS_SUFFIXES: &[&str] = &[
    "merge_prepare",
    "merge_finalize_cell_size",
    "merge_clear_buckets",
    "merge_init_owner",
    "merge_build_grid",
    "merge_find_owner",
    "merge_apply",
];

fn sim_pass_path(suffix: &str, field: &str) -> DiagnosticPath {
    DiagnosticPath::from_components(["render", SIM_PASS_PREFIX, suffix, field])
}

fn diagnostic_ms(store: &DiagnosticsStore, path: &DiagnosticPath) -> Option<f64> {
    store.get(path).and_then(|d| d.smoothed())
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ProfilingOverlay {
    pub visible: bool,
}

impl Default for ProfilingOverlay {
    fn default() -> Self {
        Self { visible: false }
    }
}

pub struct SimulationProfilingPlugin;

impl Plugin for SimulationProfilingPlugin {
    fn build(&self, app: &mut App) {
        debug_assert!(profiling_enabled(), "SimulationProfilingPlugin requires profiling_enabled()");
        app.init_resource::<AutomatedProfiling>()
            .add_systems(
                EguiPrimaryContextPass,
                draw_profiling_overlay.in_set(SimViewportSystems::Layout),
            )
            .add_systems(
                Update,
                (
                    maybe_dump_profiling_snapshot,
                    run_automated_bench,
                    run_physics_checksum,
                )
                    .chain(),
            );
    }
}

pub fn add_diagnostics_plugins(app: &mut App) {
    app.add_plugins((
        FrameTimeDiagnosticsPlugin::default(),
        RenderDiagnosticsPlugin,
    ));
}

/// Interactive overlay + pass timing (`GRAVITIUM_PROFILE=1`).
pub fn profiling_enabled() -> bool {
    env_flag("GRAVITIUM_PROFILE") || automated_profiling_active()
}

/// Unattended bench / checksum / stdout dump (also enables diagnostics).
pub fn automated_profiling_active() -> bool {
    env_flag("GRAVITIUM_BENCH")
        || env_flag("GRAVITIUM_CHECKSUM")
        || env_flag("GRAVITIUM_PROFILE_DUMP")
}

#[derive(Resource, Debug, Default)]
struct AutomatedProfiling {
    bench_frame_times_ms: Vec<f64>,
    bench_frames_seen: u32,
    checksum_frames_seen: u32,
    bench_done: bool,
    checksum_done: bool,
    last_frame_at: Option<std::time::Instant>,
}

fn count_active_bodies(snapshot: &SimulationCpuSnapshot) -> u32 {
    snapshot
        .masses
        .iter()
        .filter(|&&mass| mass > MIN_MASS)
        .count() as u32
}

fn draw_profiling_overlay(
    mut contexts: EguiContexts,
    overlay: Res<ProfilingOverlay>,
    diagnostics: Option<Res<DiagnosticsStore>>,
    settings: Res<SimulationSettings>,
    snapshot: Res<SimulationCpuSnapshot>,
) -> Result {
    if !overlay.visible {
        return Ok(());
    }
    let Some(diagnostics) = diagnostics else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;

    let frame_ms = diagnostic_ms(&diagnostics, &FrameTimeDiagnosticsPlugin::FRAME_TIME);
    let fps = diagnostic_ms(&diagnostics, &FrameTimeDiagnosticsPlugin::FPS);
    let active_bodies = count_active_bodies(&snapshot);
    let configured = settings.active_count();

    egui::Window::new("Profiling")
        .id(egui::Id::new("sim_profiling_overlay"))
        .default_pos([12.0, 12.0])
        .default_width(320.0)
        .collapsible(true)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(format!(
                "Bodies: {active_bodies} active / {configured} configured"
            ));
            if let Some(fps) = fps {
                ui.label(format!("FPS: {fps:.1}"));
            }
            if let Some(frame_ms) = frame_ms {
                ui.label(format!("Frame: {frame_ms:.2} ms"));
            }

            ui.separator();
            ui.label("Simulation compute passes (smoothed):");
            egui::Grid::new("sim_profiling_grid")
                .num_columns(3)
                .spacing([12.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Pass");
                    ui.label("CPU ms");
                    ui.label("GPU ms");
                    ui.end_row();

                    let mut sim_cpu_total = 0.0;
                    let mut sim_gpu_total = 0.0;
                    let mut sim_cpu_any = false;
                    let mut sim_gpu_any = false;
                    let mut merge_cpu_total = 0.0;
                    let mut merge_gpu_total = 0.0;
                    let mut merge_cpu_any = false;
                    let mut merge_gpu_any = false;

                    for &(suffix, label) in SIM_PASSES {
                        let cpu_path = sim_pass_path(suffix, "elapsed_cpu");
                        let gpu_path = sim_pass_path(suffix, "elapsed_gpu");
                        let cpu = diagnostic_ms(&diagnostics, &cpu_path);
                        let gpu = diagnostic_ms(&diagnostics, &gpu_path);

                        ui.label(label);
                        ui.label(format_optional_ms(cpu));
                        ui.label(format_optional_ms(gpu));
                        ui.end_row();

                        if let Some(cpu) = cpu {
                            sim_cpu_total += cpu;
                            sim_cpu_any = true;
                            if MERGE_PASS_SUFFIXES.contains(&suffix) {
                                merge_cpu_total += cpu;
                                merge_cpu_any = true;
                            }
                        }
                        if let Some(gpu) = gpu {
                            sim_gpu_total += gpu;
                            sim_gpu_any = true;
                            if MERGE_PASS_SUFFIXES.contains(&suffix) {
                                merge_gpu_total += gpu;
                                merge_gpu_any = true;
                            }
                        }
                    }

                    ui.separator();
                    ui.separator();
                    ui.separator();
                    ui.end_row();

                    ui.label("Sim total");
                    ui.label(format_total_ms(sim_cpu_total, sim_cpu_any));
                    ui.label(format_total_ms(sim_gpu_total, sim_gpu_any));
                    ui.end_row();

                    ui.label("Merge total");
                    ui.label(format_total_ms(merge_cpu_total, merge_cpu_any));
                    ui.label(format_total_ms(merge_gpu_total, merge_gpu_any));
                    ui.end_row();
                });

            ui.separator();
            ui.label("Other render passes with timing:");
            let mut other: Vec<(String, Option<f64>, Option<f64>)> = diagnostics
                .iter()
                .filter_map(|d| {
                    let path = d.path().to_string();
                    if !path.starts_with("render/")
                        || path.starts_with("render/sim/")
                        || !path.ends_with("/elapsed_cpu")
                    {
                        return None;
                    }
                    let base = path.strip_suffix("/elapsed_cpu")?;
                    let cpu = d.smoothed();
                    let gpu_path = DiagnosticPath::new(format!("{base}/elapsed_gpu"));
                    let gpu = diagnostic_ms(&diagnostics, &gpu_path);
                    let label = base.strip_prefix("render/").unwrap_or(base).to_string();
                    Some((label, cpu, gpu))
                })
                .collect();
            other.sort_by(|a, b| a.0.cmp(&b.0));

            if other.is_empty() {
                ui.label("(none yet)");
            } else {
                egui::Grid::new("render_profiling_grid")
                    .num_columns(3)
                    .spacing([12.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Pass");
                        ui.label("CPU ms");
                        ui.label("GPU ms");
                        ui.end_row();
                        for (label, cpu, gpu) in other {
                            ui.label(label);
                            ui.label(format_optional_ms(cpu));
                            ui.label(format_optional_ms(gpu));
                            ui.end_row();
                        }
                    });
            }

            ui.separator();
            ui.label(
                "CPU = command recording time. GPU = device execution time \
                 (Vulkan/DX12 only; Metal/WebGPU shows CPU only).",
            );
        });

    Ok(())
}

fn format_optional_ms(value: Option<f64>) -> String {
    value.map_or("—".to_string(), |v| format!("{v:.3}"))
}

fn format_total_ms(total: f64, any: bool) -> String {
    if any {
        format!("{total:.3}")
    } else {
        "—".to_string()
    }
}

/// One-shot stdout dump for automated profiling (`GRAVITIUM_PROFILE_DUMP=1`).
pub fn maybe_dump_profiling_snapshot(
    diagnostics: Res<DiagnosticsStore>,
    settings: Res<SimulationSettings>,
    snapshot: Res<SimulationCpuSnapshot>,
    mut dumped: Local<Vec<u32>>,
    mut frames: Local<u32>,
) {
    if !profiling_dump_enabled() {
        return;
    }
    *frames += 1;

    const SNAPSHOT_FRAMES: &[u32] = &[180, 600];
    let Some(label) = SNAPSHOT_FRAMES.iter().find(|&&target| {
        *frames == target && !dumped.iter().any(|d| *d == target)
    }) else {
        return;
    };
    dumped.push(*label);

    let active_bodies = count_active_bodies(&snapshot);
    let configured = settings.active_count();
    let frame_ms = diagnostic_ms(&diagnostics, &FrameTimeDiagnosticsPlugin::FRAME_TIME);
    let fps = diagnostic_ms(&diagnostics, &FrameTimeDiagnosticsPlugin::FPS);

    info!("=== Gravitium profiling snapshot (frame {label}) ===");
    info!("bodies_active={active_bodies} bodies_configured={configured}");
    if let (Some(fps), Some(frame_ms)) = (fps, frame_ms) {
        info!("fps={fps:.1} frame_ms={frame_ms:.2}");
    }

    let mut sim_cpu_total = 0.0;
    let mut merge_cpu_total = 0.0;
    for &(suffix, label) in SIM_PASSES {
        let cpu = diagnostic_ms(&diagnostics, &sim_pass_path(suffix, "elapsed_cpu"));
        if let Some(cpu) = cpu {
            sim_cpu_total += cpu;
            if MERGE_PASS_SUFFIXES.contains(&suffix) {
                merge_cpu_total += cpu;
            }
        }
        info!(
            "pass={label} cpu_ms={}",
            cpu.map(|v| format!("{v:.3}")).unwrap_or_else(|| "—".into())
        );
    }
    info!("sim_cpu_total_ms={sim_cpu_total:.3} merge_cpu_total_ms={merge_cpu_total:.3}");

    for diagnostic in diagnostics.iter() {
        let path = diagnostic.path().to_string();
        if path.starts_with("render/sim/") || !path.ends_with("/elapsed_cpu") {
            continue;
        }
        let label = path.strip_prefix("render/").unwrap_or(&path);
        let base = label.strip_suffix("/elapsed_cpu").unwrap_or(label);
        if let Some(cpu) = diagnostic.smoothed() {
            info!("render_pass={base} cpu_ms={cpu:.3}");
        }
    }
    info!("=== end profiling snapshot ===");
}

fn run_automated_bench(
    settings: Res<SimulationSettings>,
    mut profiling: ResMut<AutomatedProfiling>,
    mut exit: MessageWriter<AppExit>,
) {
    if !bench_enabled() {
        return;
    }
    if profiling.bench_done {
        return;
    }

    let target_frames = bench_target_frames();
    let warmup_frames = bench_warmup_frames();
    let now = std::time::Instant::now();

    if let Some(last) = profiling.last_frame_at {
        let frame_ms = now.duration_since(last).as_secs_f64() * 1000.0;
        if profiling.bench_frames_seen > warmup_frames {
            profiling.bench_frame_times_ms.push(frame_ms);
        }
    }
    profiling.last_frame_at = Some(now);
    profiling.bench_frames_seen += 1;

    if profiling.bench_frames_seen % 50 == 0 {
        eprintln!("bench progress: frame {}", profiling.bench_frames_seen);
    }

    if profiling.bench_frames_seen <= target_frames + warmup_frames {
        return;
    }

    profiling.bench_done = true;
    let active_count = settings.active_count();
    let seed = settings.initial.seed;
    let samples = &profiling.bench_frame_times_ms;
    let avg = samples.iter().sum::<f64>() / samples.len().max(1) as f64;
    let mut sorted = samples.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p95_idx = ((sorted.len() as f64) * 0.95).ceil() as usize;
    let p95 = sorted[p95_idx.saturating_sub(1).min(sorted.len().saturating_sub(1))];

    let json = format!(
        "{{\"kind\":\"bench\",\"seed\":{seed},\"active_count\":{active_count},\"frames\":{},\"warmup_frames\":{warmup_frames},\"frame_ms_avg\":{avg:.2},\"frame_ms_p95\":{p95:.2}}}",
        samples.len()
    );
    eprintln!("{json}");
    let _ = std::fs::write("/tmp/gravitium_bench.json", &json);
    exit.write(AppExit::Success);
}

fn run_physics_checksum(
    snapshot: Res<SimulationCpuSnapshot>,
    settings: Res<SimulationSettings>,
    mut profiling: ResMut<AutomatedProfiling>,
    mut exit: MessageWriter<AppExit>,
) {
    if !checksum_enabled() {
        return;
    }
    if profiling.checksum_done {
        return;
    }

    let target_frame = checksum_target_frame();
    profiling.checksum_frames_seen += 1;
    if profiling.checksum_frames_seen < target_frame {
        return;
    }
    if !snapshot.ready {
        return;
    }

    profiling.checksum_done = true;
    let active_count = settings.active_count() as usize;
    let seed = settings.initial.seed;
    let hash = physics_state_hash(&snapshot, active_count);

    info!(
        "{{\"kind\":\"checksum\",\"seed\":{seed},\"active_count\":{active_count},\"frame\":{target_frame},\"hash\":\"{hash:016x}\"}}"
    );

    if bench_enabled() {
        return;
    }
    exit.write(AppExit::Success);
}

pub fn physics_state_hash(snapshot: &SimulationCpuSnapshot, active_count: usize) -> u64 {
    let mut hasher = DefaultHasher::new();
    let count = active_count.min(snapshot.positions.len()).min(snapshot.masses.len());
    for i in 0..count {
        for component in snapshot.positions[i].to_array() {
            component.to_bits().hash(&mut hasher);
        }
        snapshot.masses[i].to_bits().hash(&mut hasher);
    }
    hasher.finish()
}

fn profiling_dump_enabled() -> bool {
    env_flag("GRAVITIUM_PROFILE_DUMP")
}

fn bench_enabled() -> bool {
    env_flag("GRAVITIUM_BENCH")
}

fn checksum_enabled() -> bool {
    env_flag("GRAVITIUM_CHECKSUM")
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn bench_target_frames() -> u32 {
    env_u32("GRAVITIUM_BENCH_FRAMES", 600)
}

fn bench_warmup_frames() -> u32 {
    env_u32("GRAVITIUM_BENCH_WARMUP", 180)
}

fn checksum_target_frame() -> u32 {
    env_u32("GRAVITIUM_CHECKSUM_FRAMES", 600)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_state_hash_is_stable_for_known_input() {
        let snapshot = SimulationCpuSnapshot {
            positions: vec![Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)],
            masses: vec![1.0, 2.0],
            ready: true,
        };
        let h1 = physics_state_hash(&snapshot, 2);
        let h2 = physics_state_hash(&snapshot, 2);
        assert_eq!(h1, h2);
    }
}
