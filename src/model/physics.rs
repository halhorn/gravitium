use super::constants::{
    MERGE_CELL_MIN_SIZE, MERGE_CELL_RADIUS_SAFETY, MERGE_MAX_RADIUS, MERGE_RADIUS_FACTOR,
    MERGE_RADIUS_FACTOR_MAX, MERGE_RADIUS_FACTOR_MIN, SOFTENING, SOFTENING_MAX, SOFTENING_MIN,
};

/// How merge spatial-hash cell size is chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MergeCellSizePolicy {
    /// Legacy fixed cap from `MERGE_MAX_RADIUS`.
    ConservativeFixed,
    /// Total-mass envelope at simulation start / restart (CPU).
    #[default]
    InitialMassEnvelope,
    /// Per-merge-iteration max radius on GPU (`prepare` + `finalize_cell_size`).
    AdaptivePerPrepare,
}

/// Runtime physics parameters (defaults match legacy compile-time constants).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsSettings {
    pub softening: f32,
    pub merge_radius_factor: f32,
    pub merge_cell_policy: MergeCellSizePolicy,
    pub merge_cell_min_size: f32,
    pub merge_cell_radius_safety: f32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            softening: SOFTENING,
            merge_radius_factor: MERGE_RADIUS_FACTOR,
            merge_cell_policy: MergeCellSizePolicy::default(),
            merge_cell_min_size: MERGE_CELL_MIN_SIZE,
            merge_cell_radius_safety: MERGE_CELL_RADIUS_SAFETY,
        }
    }
}

impl PhysicsSettings {
    pub fn softening_sq(&self) -> f32 {
        self.softening * self.softening
    }

    /// Cell size from a radius cap and current merge settings.
    pub fn merge_cell_size_from_radius_cap(&self, radius_cap: f32) -> f32 {
        let safe_radius = radius_cap.max(0.0) * self.merge_cell_radius_safety;
        (2.0 * safe_radius * self.merge_radius_factor).max(self.merge_cell_min_size)
    }

    /// Inverse cell size from a radius cap.
    pub fn merge_inv_cell_size_from_radius_cap(&self, radius_cap: f32) -> f32 {
        1.0 / self.merge_cell_size_from_radius_cap(radius_cap)
    }

    /// Legacy fixed cell size (`MERGE_MAX_RADIUS` cap).
    pub fn conservative_merge_inv_cell_size(&self) -> f32 {
        self.merge_inv_cell_size_from_radius_cap(MERGE_MAX_RADIUS)
    }

    /// Back-compat alias for conservative fixed sizing.
    pub fn merge_inv_cell_size(&self) -> f32 {
        self.conservative_merge_inv_cell_size()
    }

    pub fn clamped(self) -> Self {
        Self {
            softening: self.softening.clamp(SOFTENING_MIN, SOFTENING_MAX),
            merge_radius_factor: self
                .merge_radius_factor
                .clamp(MERGE_RADIUS_FACTOR_MIN, MERGE_RADIUS_FACTOR_MAX),
            merge_cell_policy: self.merge_cell_policy,
            merge_cell_min_size: self.merge_cell_min_size.max(1e-4),
            merge_cell_radius_safety: self.merge_cell_radius_safety.max(1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conservative_cell_size_matches_legacy_formula() {
        let physics = PhysicsSettings {
            merge_cell_policy: MergeCellSizePolicy::ConservativeFixed,
            ..Default::default()
        };
        let cell = physics.merge_cell_size_from_radius_cap(MERGE_MAX_RADIUS);
        assert!((cell - 10.0).abs() < 1e-4);
    }

    #[test]
    fn small_radius_cap_yields_small_cell() {
        let physics = PhysicsSettings::default();
        let cell = physics.merge_cell_size_from_radius_cap(0.005);
        assert!((cell - 0.2).abs() < 1e-4);
    }
}
