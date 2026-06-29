use bevy::prelude::*;
use bevy::render::storage::ShaderStorageBuffer;

use crate::model::{generate_initial_state, BodyArrays};

use super::commands::{SimulationCommand, SimulationSpawned};
use super::gpu::SimulationGpuBuffers;
use super::merge_cell::MergeCellCpuState;
use super::playback::PlaybackState;
use super::settings::SimulationSettings;
use super::upload::{queue_upload, PendingSimulationUpload};

/// One-shot startup: generate state and create GPU buffers.
pub fn spawn_initial_simulation(
    mut commands: Commands,
    settings: Res<SimulationSettings>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut spawned: MessageWriter<SimulationSpawned>,
) {
    let bodies = generate_initial_state(&settings.initial, &settings.physics, &settings.force);
    commands.insert_resource(MergeCellCpuState::from_masses(
        &settings.physics,
        &bodies.masses,
        bodies.active_count as usize,
    ));
    install_simulation_state(&mut commands, &mut buffers, &bodies, &mut spawned, false);
}

/// Apply a restart: reset time, regenerate bodies, queue GPU upload, keep running.
pub fn restart_simulation(
    mut commands: MessageReader<SimulationCommand>,
    mut playback: ResMut<PlaybackState>,
    settings: Res<SimulationSettings>,
    gpu: Option<Res<SimulationGpuBuffers>>,
    mut pending_upload: ResMut<PendingSimulationUpload>,
    mut merge_cell: ResMut<MergeCellCpuState>,
    mut spawned: MessageWriter<SimulationSpawned>,
) {
    if !commands
        .read()
        .any(|command| matches!(command, SimulationCommand::Restart))
    {
        return;
    }

    let Some(_gpu) = gpu else {
        return;
    };

    playback.accumulated_sim_time = 0.0;

    let bodies = generate_initial_state(&settings.initial, &settings.physics, &settings.force);
    *merge_cell = MergeCellCpuState::from_masses(
        &settings.physics,
        &bodies.masses,
        bodies.active_count as usize,
    );
    queue_upload(&mut pending_upload, &bodies);
    write_spawned(&mut spawned, &bodies, true);
}

fn install_simulation_state(
    commands: &mut Commands,
    buffers: &mut Assets<ShaderStorageBuffer>,
    bodies: &BodyArrays,
    spawned: &mut MessageWriter<SimulationSpawned>,
    pending_readback: bool,
) {
    let (positions, velocities, masses, accelerations) =
        super::upload::body_arrays_to_vec4(bodies);

    let gpu = SimulationGpuBuffers::new(buffers, positions, velocities, masses, accelerations);
    commands.insert_resource(gpu);

    write_spawned(spawned, bodies, pending_readback);
}

fn write_spawned(
    spawned: &mut MessageWriter<SimulationSpawned>,
    bodies: &BodyArrays,
    pending_readback: bool,
) {
    spawned.write(SimulationSpawned {
        positions: bodies
            .positions
            .iter()
            .map(|p| Vec3::new(p[0], p[1], p[2]))
            .collect(),
        masses: bodies.masses.clone(),
        pending_readback,
    });
}
