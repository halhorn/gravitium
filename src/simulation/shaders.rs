use bevy::{asset::uuid_handle, prelude::*};

use crate::model::constants::{BODY_COUNT, MERGE_FLASH_FRAMES};
use crate::simulation::gpu::params::{
    MERGE_SCRATCH_METADATA_INDEX, MERGE_SCRATCH_PARTIAL_RADIUS_OFFSET,
    MERGE_SCRATCH_VEL_RADIUS_OFFSET,
};

const GRAVITY_WGSL: &str = include_str!("../../assets/shaders/gravity.wgsl");
const INTEGRATE_WGSL: &str = include_str!("../../assets/shaders/integrate.wgsl");
const BODIES_WGSL: &str = include_str!("../../assets/shaders/bodies.wgsl");
const COLORS_WGSL: &str = include_str!("../../assets/shaders/colors.wgsl");
const MERGE_WGSL: &str = include_str!("../../assets/shaders/merge.wgsl");

fn inject_shader_constants(source: &str) -> String {
    source
        .replace("#{MERGE_FLASH_FRAMES}", &MERGE_FLASH_FRAMES.to_string())
        .replace("#{BODY_COUNT}", &BODY_COUNT.to_string())
        .replace(
            "#{MERGE_SCRATCH_VEL_RADIUS_OFFSET}",
            &MERGE_SCRATCH_VEL_RADIUS_OFFSET.to_string(),
        )
        .replace(
            "#{MERGE_SCRATCH_METADATA_INDEX}",
            &MERGE_SCRATCH_METADATA_INDEX.to_string(),
        )
        .replace(
            "#{MERGE_SCRATCH_PARTIAL_RADIUS_OFFSET}",
            &MERGE_SCRATCH_PARTIAL_RADIUS_OFFSET.to_string(),
        )
}

pub const GRAVITY_SHADER: Handle<Shader> = uuid_handle!("a8c31e42-1f0b-4d2a-9e3c-7b5a6d8e9f01");
pub const INTEGRATE_SHADER: Handle<Shader> = uuid_handle!("b9d42f53-2a1c-5e3b-0f4d-8c6b7e0f1a02");
pub const BODIES_SHADER: Handle<Shader> = uuid_handle!("f3a91c2e-8b4d-4a1e-9c2f-1d8e5a6b7c0d");
pub const COLORS_SHADER: Handle<Shader> = uuid_handle!("d5f93b75-4c3e-475d-2b6f-0e4d9c1f3b04");
pub const MERGE_SHADER: Handle<Shader> = uuid_handle!("c4e82a64-3b2d-6f4c-1a5e-9d7c8b0e2f03");

/// Embeds WGSL at compile time (wasm-safe; no runtime asset fetch).
pub fn register_simulation_shaders(mut shaders: ResMut<Assets<Shader>>) {
    let _ = shaders.insert(
        GRAVITY_SHADER.id(),
        Shader::from_wgsl(GRAVITY_WGSL, "shaders/gravity.wgsl"),
    );
    let _ = shaders.insert(
        INTEGRATE_SHADER.id(),
        Shader::from_wgsl(INTEGRATE_WGSL, "shaders/integrate.wgsl"),
    );
    let _ = shaders.insert(
        BODIES_SHADER.id(),
        Shader::from_wgsl(BODIES_WGSL, "shaders/bodies.wgsl"),
    );
    let _ = shaders.insert(
        COLORS_SHADER.id(),
        Shader::from_wgsl(
            inject_shader_constants(COLORS_WGSL),
            "shaders/colors.wgsl",
        ),
    );
    let _ = shaders.insert(
        MERGE_SHADER.id(),
        Shader::from_wgsl(
            inject_shader_constants(MERGE_WGSL),
            "shaders/merge.wgsl",
        ),
    );
}
