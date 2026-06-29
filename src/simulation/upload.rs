use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

use crate::model::constants::{BODY_COUNT, MERGE_BUCKET_COUNT};
use crate::simulation::gpu::params::MERGE_SCRATCH_LEN;
use crate::model::BodyArrays;

/// Body state waiting to be written into existing GPU buffers (Render world).
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct PendingSimulationUpload {
    pub payload: Option<SimulationUploadPayload>,
}

#[derive(Clone)]
pub struct SimulationUploadPayload {
    pub positions: Vec<Vec4>,
    pub velocities: Vec<Vec4>,
    pub masses: Vec<f32>,
    pub accelerations: Vec<Vec4>,
    pub accelerations_new: Vec<Vec4>,
    pub merge_bucket_heads: Vec<u32>,
    pub merge_aux: Vec<u32>,
    pub merge_owner: Vec<u32>,
    pub merge_scratch: Vec<Vec4>,
}

/// Convert model arrays to Bevy `Vec4` for GPU upload.
pub fn body_arrays_to_vec4(bodies: &BodyArrays) -> (Vec<Vec4>, Vec<Vec4>, Vec<f32>, Vec<Vec4>) {
    let positions = bodies
        .positions
        .iter()
        .map(|p| Vec4::new(p[0], p[1], p[2], p[3]))
        .collect();
    let velocities = bodies
        .velocities
        .iter()
        .map(|v| Vec4::new(v[0], v[1], v[2], v[3]))
        .collect();
    let masses = bodies.masses.clone();
    let accelerations = bodies
        .accelerations
        .iter()
        .map(|a| Vec4::new(a[0], a[1], a[2], a[3]))
        .collect();
    (positions, velocities, masses, accelerations)
}

impl SimulationUploadPayload {
    pub fn from_bodies(bodies: &BodyArrays) -> Self {
        let (positions, velocities, masses, accelerations) = body_arrays_to_vec4(bodies);

        let merge_aux = vec![0u32; BODY_COUNT * 2];

        Self {
            positions,
            velocities,
            masses,
            accelerations,
            accelerations_new: vec![Vec4::ZERO; BODY_COUNT],
            merge_bucket_heads: vec![u32::MAX; MERGE_BUCKET_COUNT],
            merge_aux,
            merge_owner: vec![BODY_COUNT as u32; BODY_COUNT],
            merge_scratch: vec![Vec4::ZERO; MERGE_SCRATCH_LEN],
        }
    }
}

/// Queue an in-place GPU upload for the next render frame.
pub fn queue_upload(pending: &mut PendingSimulationUpload, bodies: &BodyArrays) {
    pending.payload = Some(SimulationUploadPayload::from_bodies(bodies));
}
