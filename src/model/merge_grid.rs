use super::body::physical_radius;
use super::constants::{MERGE_MAX_RADIUS, MIN_MASS};
use super::physics::{MergeCellSizePolicy, PhysicsSettings};

/// CPU-side merge spatial-hash sizing (pure logic, no Bevy/GPU).
pub struct MergeGridSizer;

impl MergeGridSizer {
    /// Mass → physical radius (`SUN_RADIUS_AU * mass^(1/3)`).
    #[inline]
    pub fn radius_from_mass(mass: f32) -> f32 {
        physical_radius(mass)
    }

    /// Legacy conservative radius cap (`MERGE_MAX_RADIUS`).
    #[inline]
    pub fn conservative_radius_cap() -> f32 {
        MERGE_MAX_RADIUS
    }

    /// Safe upper bound on stellar radius if all active mass coalesces into one body.
    pub fn initial_mass_envelope_radius_cap(masses: &[f32], active_count: usize) -> f32 {
        let total_mass: f32 = masses
            .iter()
            .take(active_count)
            .filter(|&&m| m > MIN_MASS)
            .sum();
        if total_mass <= MIN_MASS {
            return Self::conservative_radius_cap();
        }
        Self::radius_from_mass(total_mass)
    }

    /// CPU-side radius cap for the current policy, if any.
    pub fn cpu_radius_cap(
        physics: &PhysicsSettings,
        masses: &[f32],
        active_count: usize,
    ) -> Option<f32> {
        match physics.merge_cell_policy {
            MergeCellSizePolicy::ConservativeFixed => Some(Self::conservative_radius_cap()),
            MergeCellSizePolicy::InitialMassEnvelope => {
                Some(Self::initial_mass_envelope_radius_cap(masses, active_count))
            }
            MergeCellSizePolicy::AdaptivePerPrepare => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PhysicsSettings;

    #[test]
    fn envelope_cap_uses_total_active_mass() {
        let masses = vec![1.0, 0.01, 0.01, 0.0];
        let cap = MergeGridSizer::initial_mass_envelope_radius_cap(&masses, 3);
        let expected = MergeGridSizer::radius_from_mass(1.02);
        assert!((cap - expected).abs() < 1e-6);
    }

    #[test]
    fn conservative_policy_returns_fixed_cap() {
        let mut physics = PhysicsSettings::default();
        physics.merge_cell_policy = MergeCellSizePolicy::ConservativeFixed;
        let cap = MergeGridSizer::cpu_radius_cap(&physics, &[], 0).unwrap();
        assert_eq!(cap, MERGE_MAX_RADIUS);
    }

    #[test]
    fn adaptive_policy_has_no_cpu_cap() {
        let mut physics = PhysicsSettings::default();
        physics.merge_cell_policy = MergeCellSizePolicy::AdaptivePerPrepare;
        assert!(MergeGridSizer::cpu_radius_cap(&physics, &[], 0).is_none());
    }
}
