use std::f32::consts::PI;

pub const WORKGROUP_SIZE: u32 = 256;

/// Bodies with `mass <= MIN_MASS` are inactive (merged away).
pub const MIN_MASS: f32 = 1e-8;

/// Gravitational constant in AU³/(M☉·yr²).
pub const G_MIN: f32 = 1.0;
pub const G: f32 = 4.0 * PI * PI;
pub const G_MAX: f32 = 100.0;

/// Plummer softening length (AU).
pub const SOFTENING_MIN: f32 = 0.001;
pub const SOFTENING: f32 = 0.01;
pub const SOFTENING_MAX: f32 = 0.1;

/// Fraction of combined radii at which bodies merge (legacy `merger.rs`).
pub const MERGE_RADIUS_FACTOR_MIN: f32 = 0.01;
pub const MERGE_RADIUS_FACTOR: f32 = 20.0;
pub const MERGE_RADIUS_FACTOR_MAX: f32 = 100.0;

/// Frames to fade merge flash back to mass-based body color.
pub const MERGE_FLASH_FRAMES: u32 = 60;

/// Spatial hash buckets for the merge pass (32k balances chain length vs clear cost).
pub const MERGE_BUCKET_COUNT: usize = 32_768;

/// Merge sub-pipeline runs per simulation frame (clears grid between iterations).
pub const MERGE_ITERATIONS_PER_FRAME: u32 = 2;

/// Conservative max body radius (AU) for merge grid; covers `STAR_MASS_MAX` at `SUN_RADIUS_AU`.
pub const MERGE_MAX_RADIUS: f32 = 0.25;

/// Minimum spatial-hash cell size (AU) for merge broad-phase.
pub const MERGE_CELL_MIN_SIZE: f32 = 0.01;

/// Safety multiplier applied to radius cap before computing merge cell size.
pub const MERGE_CELL_RADIUS_SAFETY: f32 = 1.0;

/// Solar radius in AU (696,000 km / 1 AU).
pub const SUN_RADIUS_AU: f32 = 696_000.0 / 149_597_870.7;

/// Rendering multiplier for star spheres (physics radii stay at `SUN_RADIUS_AU`).
pub const STAR_VISUAL_SCALE_MIN: f32 = 1.0;
/// Default ≈ old `0.5 * M^(1/3)` appearance for a 1 M☉ star.
pub const STAR_VISUAL_SCALE: f32 = 50.0;
pub const STAR_VISUAL_SCALE_MAX: f32 = 1000.0;

/// Minimum rendered star radius in AU (slider lower bound).
pub const MIN_STAR_VISUAL_SCALE_MIN: f32 = 0.001;
pub const MIN_STAR_VISUAL_SCALE: f32 = 0.02;
pub const MIN_STAR_VISUAL_SCALE_MAX: f32 = 0.5;

/// Initial-condition UI / validation ranges (Phase 3).
/// RNG seed: up to 8 decimal digits.
pub const SEED_MAX: u64 = 99_999_999;
pub const SEED: u64 = 12_345_678;

/// Central star count (0 = disk-only, no bulge stars).
pub const N_STARS_MIN: u32 = 0;
pub const N_STARS: u32 = 1;
pub const N_STARS_MAX: u32 = 4;

pub const ACTIVE_COUNT_MIN: u32 = 2;
/// Default active body count at startup.
pub const ACTIVE_COUNT: u32 = 10_000;
/// Maximum active bodies (UI slider upper bound).
pub const ACTIVE_COUNT_MAX: u32 = 20_000;
/// GPU/CPU buffer length; equals `ACTIVE_COUNT_MAX` (one slot per possible body).
pub const BODY_COUNT: usize = ACTIVE_COUNT_MAX as usize;

pub const STAR_MASS_MIN: f32 = 0.1;
pub const STAR_MASS: f32 = 100.0;
pub const STAR_MASS_MAX: f32 = 100000.0;

/// Disk star mass uniform range [min, max] in M☉ (slider limits).
pub const DISK_MASS_LIMIT_MIN: f32 = 0.000001;
pub const DISK_MASS_LIMIT_MAX: f32 = 1000.0;
/// Default disk mass uniform range.
pub const DISK_MASS_MIN: f32 = 0.002;
pub const DISK_MASS_MAX: f32 = 0.02;

pub const DISK_R_MIN: f32 = 0.001;
pub const DISK_R_MAX: f32 = 1000.0;
/// Default disk inner / outer radius (AU).
pub const DISK_R_INNER: f32 = 0.01;
pub const DISK_R_OUTER: f32 = 60.0;
/// Max elevation from the disk plane (degrees). At 90° the disk fills a sphere.
pub const DISK_ELEVATION_DEG_MAX: f32 = 90.0;
/// Default half-opening angle from the disk plane (degrees).
pub const DISK_ELEVATION_DEG: f32 = 1.0;
pub const V_PERTURBATION: f32 = 0.5;
pub const V_PERTURBATION_MAX: f32 = 2.0;

/// Force-law polynomial term limits (Phase 4).
pub const FORCE_EXPONENT_MIN: i32 = -5;
pub const FORCE_EXPONENT_MAX: i32 = 2;
pub const FORCE_COEFFICIENT_MIN: f32 = 1e-6;
pub const FORCE_COEFFICIENT_MAX: f32 = 1000.0;
/// Default coefficient for a newly added force term (kept small so the extra term
/// does not immediately destabilize the running simulation).
pub const NEW_FORCE_TERM_COEFFICIENT: f32 = 1e-4;
