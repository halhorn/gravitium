use bevy::render::render_resource::ShaderType;

use crate::model::constants::{BODY_COUNT, MIN_MASS, WORKGROUP_SIZE};
use crate::model::force::MAX_FORCE_TERMS;
use crate::model::merge_grid::MergeGridSizer;
use crate::model::physics::MergeCellSizePolicy;
use crate::simulation::config::SimulationConfig;
use crate::simulation::merge_cell::MergeCellCpuState;
use crate::simulation::settings::SimulationSettings;

pub const MERGE_CELL_MODE_FIXED: u32 = 0;
pub const MERGE_CELL_MODE_GPU_ADAPTIVE: u32 = 1;

pub const MERGE_SCRATCH_BODY_OFFSET: usize = 0;
pub const MERGE_SCRATCH_VEL_RADIUS_OFFSET: usize = BODY_COUNT;
pub const MERGE_SCRATCH_METADATA_INDEX: usize = BODY_COUNT * 2;
pub const MERGE_SCRATCH_PARTIAL_RADIUS_OFFSET: usize = BODY_COUNT * 2 + 1;

pub const MAX_MERGE_WORKGROUPS: usize = BODY_COUNT.div_ceil(WORKGROUP_SIZE as usize);
pub const MERGE_SCRATCH_LEN: usize = BODY_COUNT * 2 + 1 + MAX_MERGE_WORKGROUPS;

#[derive(Clone, Copy, ShaderType, PartialEq)]
pub struct GpuForceTerm {
    pub sign: i32,
    pub exponent: i32,
    pub coefficient: f32,
    pub _pad: u32,
}

#[derive(Clone, Copy, ShaderType, PartialEq)]
pub struct GravityParams {
    pub n: u32,
    pub term_count: u32,
    pub softening_sq: f32,
    pub min_mass: f32,
    pub terms: [GpuForceTerm; 8],
}

#[derive(Clone, Copy, ShaderType, PartialEq)]
pub struct IntegrateParams {
    pub n: u32,
    pub dt: f32,
    pub min_mass: f32,
    pub _pad: f32,
}

#[derive(Clone, Copy, ShaderType, PartialEq)]
pub struct MergeParams {
    pub n: u32,
    pub merge_radius_factor: f32,
    pub inv_cell_size: f32,
    pub min_mass: f32,
    pub cell_size_mode: u32,
    pub radius_partial_count: u32,
    pub merge_cell_min_size: f32,
    pub merge_cell_radius_safety: f32,
}

impl GravityParams {
    pub fn from_settings(settings: &SimulationSettings) -> Self {
        let force = settings.force.clone().clamped();
        let mut terms = [GpuForceTerm {
            sign: 0,
            exponent: 0,
            coefficient: 0.0,
            _pad: 0,
        }; MAX_FORCE_TERMS];

        for (i, term) in force
            .terms
            .iter()
            .take(force.term_count as usize)
            .enumerate()
        {
            terms[i] = GpuForceTerm {
                sign: term.sign as i32,
                exponent: term.exponent,
                coefficient: term.coefficient,
                _pad: 0,
            };
        }

        Self {
            n: settings.active_count(),
            term_count: force.term_count as u32,
            softening_sq: settings.physics.softening_sq(),
            min_mass: MIN_MASS,
            terms,
        }
    }
}

impl IntegrateParams {
    pub fn from_settings(settings: &SimulationSettings, config: &SimulationConfig) -> Self {
        Self {
            n: settings.active_count(),
            dt: config.dt(),
            min_mass: MIN_MASS,
            _pad: 0.0,
        }
    }
}

impl MergeParams {
    pub fn from_settings(settings: &SimulationSettings, merge_cell: &MergeCellCpuState) -> Self {
        let physics = settings.physics;
        let active_count = settings.active_count();
        let radius_partial_count = active_count.div_ceil(WORKGROUP_SIZE);

        let (cell_size_mode, inv_cell_size) = match physics.merge_cell_policy {
            MergeCellSizePolicy::AdaptivePerPrepare => (
                MERGE_CELL_MODE_GPU_ADAPTIVE,
                physics.conservative_merge_inv_cell_size(),
            ),
            MergeCellSizePolicy::ConservativeFixed => (
                MERGE_CELL_MODE_FIXED,
                physics.conservative_merge_inv_cell_size(),
            ),
            MergeCellSizePolicy::InitialMassEnvelope => {
                let cap = merge_cell
                    .radius_cap
                    .unwrap_or_else(MergeGridSizer::conservative_radius_cap);
                (
                    MERGE_CELL_MODE_FIXED,
                    physics.merge_inv_cell_size_from_radius_cap(cap),
                )
            }
        };

        Self {
            n: active_count,
            merge_radius_factor: physics.merge_radius_factor,
            inv_cell_size,
            min_mass: MIN_MASS,
            cell_size_mode,
            radius_partial_count,
            merge_cell_min_size: physics.merge_cell_min_size,
            merge_cell_radius_safety: physics.merge_cell_radius_safety,
        }
    }
}

#[derive(Clone, Copy, ShaderType, PartialEq)]
pub struct ColorsParams {
    pub n: u32,
    pub min_mass: f32,
    pub _pad0: f32,
    pub _pad1: f32,
}

impl ColorsParams {
    pub fn from_settings(settings: &SimulationSettings) -> Self {
        Self {
            n: settings.active_count(),
            min_mass: MIN_MASS,
            _pad0: 0.0,
            _pad1: 0.0,
        }
    }
}
