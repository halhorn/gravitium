use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

/// CPU-computed merge radius cap for non-adaptive cell policies.
#[derive(Resource, Clone, Copy, Default, ExtractResource)]
pub struct MergeCellCpuState {
    pub radius_cap: Option<f32>,
}

impl MergeCellCpuState {
    pub fn from_masses(
        physics: &crate::model::PhysicsSettings,
        masses: &[f32],
        active_count: usize,
    ) -> Self {
        Self {
            radius_cap: crate::model::MergeGridSizer::cpu_radius_cap(
                physics,
                masses,
                active_count,
            ),
        }
    }
}
