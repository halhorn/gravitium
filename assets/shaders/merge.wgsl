// Parallel merge (≤8 storage buffers for WebGPU).
// Scratch layout:
//   [0 .. BODY_COUNT) pos.xyz + mass
//   [BODY_COUNT .. BODY_COUNT*2) vel.xyz + radius
//   [BODY_COUNT*2] metadata (radius_cap, inv_cell_size)
//   [BODY_COUNT*2 + 1 .. + 1 + workgroups) per-workgroup radius partials
// merge_aux: [0..n) bucket_next, [n..2n) merge_flash.

const INVALID: u32 = 0xFFFFFFFFu;
const MERGE_FLASH_FRAMES: u32 = #{MERGE_FLASH_FRAMES}u;
const ABSORBED_THIS_PASS: u32 = 1u;
const BODY_COUNT: u32 = #{BODY_COUNT}u;
const SCRATCH_VEL_RADIUS_OFFSET: u32 = #{MERGE_SCRATCH_VEL_RADIUS_OFFSET}u;
const SCRATCH_METADATA_INDEX: u32 = #{MERGE_SCRATCH_METADATA_INDEX}u;
const SCRATCH_PARTIAL_RADIUS_OFFSET: u32 = #{MERGE_SCRATCH_PARTIAL_RADIUS_OFFSET}u;
const MERGE_CELL_MODE_FIXED: u32 = 0u;
const MERGE_CELL_MODE_GPU_ADAPTIVE: u32 = 1u;

struct Params {
    n: u32,
    merge_radius_factor: f32,
    inv_cell_size: f32,
    min_mass: f32,
    cell_size_mode: u32,
    radius_partial_count: u32,
    merge_cell_min_size: f32,
    merge_cell_radius_safety: f32,
}

@group(0) @binding(0) var<storage, read_write> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> masses: array<f32>;
@group(0) @binding(3) var<storage, read_write> accelerations: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> scratch: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> bucket_heads: array<atomic<u32>>;
@group(0) @binding(6) var<storage, read_write> merge_aux: array<u32>;
@group(0) @binding(7) var<storage, read_write> merge_owner: array<atomic<u32>>;
@group(0) @binding(8) var<uniform> params: Params;

var<workgroup> wg_max_radius: f32;

fn bucket_next(i: u32) -> u32 {
    return merge_aux[i];
}

fn set_bucket_next(i: u32, value: u32) {
    merge_aux[i] = value;
}

fn absorbed(j: u32) -> u32 {
    return merge_aux[params.n + j];
}

fn set_absorbed(j: u32, value: u32) {
    merge_aux[params.n + j] = value;
}

fn flash_counter(v: u32) -> u32 {
    return v >> 1u;
}

fn absorbed_this_pass(v: u32) -> bool {
    return (v & ABSORBED_THIS_PASS) != 0u;
}

fn encode_flash(flash: u32, absorbed: bool) -> u32 {
    var v = flash << 1u;
    if (absorbed) {
        v |= ABSORBED_THIS_PASS;
    }
    return v;
}

fn snap_pos(i: u32) -> vec3<f32> {
    return scratch[i].xyz;
}

fn snap_mass(i: u32) -> f32 {
    return scratch[i].w;
}

fn snap_vel(i: u32) -> vec3<f32> {
    return scratch[SCRATCH_VEL_RADIUS_OFFSET + i].xyz;
}

fn snap_radius(i: u32) -> f32 {
    return scratch[SCRATCH_VEL_RADIUS_OFFSET + i].w;
}

const SUN_RADIUS_AU: f32 = 696000.0 / 149597870.7;

fn physical_radius_from_mass(mass: f32) -> f32 {
    return SUN_RADIUS_AU * pow(max(mass, 0.0), 1.0 / 3.0);
}

fn radius_cap_to_inv_cell_size(radius_cap: f32) -> f32 {
    let safe_radius = max(radius_cap * params.merge_cell_radius_safety, 0.0);
    let cell_size = max(
        2.0 * safe_radius * params.merge_radius_factor,
        params.merge_cell_min_size,
    );
    return 1.0 / cell_size;
}

fn adaptive_inv_cell_size() -> f32 {
    return scratch[SCRATCH_METADATA_INDEX].y;
}

fn current_inv_cell_size() -> f32 {
    if (params.cell_size_mode == MERGE_CELL_MODE_GPU_ADAPTIVE) {
        return adaptive_inv_cell_size();
    }
    return params.inv_cell_size;
}

fn hash_cell(cx: i32, cy: i32, cz: i32) -> u32 {
    let hx = bitcast<u32>(cx);
    let hy = bitcast<u32>(cy);
    let hz = bitcast<u32>(cz);
    let h = hx * 73856093u ^ hy * 19349663u ^ hz * 83492791u;
    return h % arrayLength(&bucket_heads);
}

fn cell_coords(pos: vec3<f32>) -> vec3<i32> {
    let s = current_inv_cell_size();
    return vec3<i32>(
        i32(floor(pos.x * s)),
        i32(floor(pos.y * s)),
        i32(floor(pos.z * s)),
    );
}

fn mergeable(i: u32, j: u32) -> bool {
    if (j <= i || absorbed_this_pass(absorbed(j)) || snap_mass(j) <= params.min_mass) {
        return false;
    }
    if (snap_mass(i) <= params.min_mass) {
        return false;
    }
    let dist = length(snap_pos(i) - snap_pos(j));
    let touch = (snap_radius(i) + snap_radius(j)) * params.merge_radius_factor;
    return dist < touch;
}

fn absorb(i: u32, j: u32) {
    let mi = snap_mass(i);
    let mj = snap_mass(j);
    let new_mass = mi + mj;
    velocities[i] = vec4<f32>((snap_vel(i) * mi + snap_vel(j) * mj) / new_mass, 0.0);
    positions[i] = vec4<f32>((snap_pos(i) * mi + snap_pos(j) * mj) / new_mass, 0.0);
    masses[i] = new_mass;
    masses[j] = 0.0;
    accelerations[i] = vec4<f32>(0.0);
    set_absorbed(i, encode_flash(MERGE_FLASH_FRAMES, false));
    set_absorbed(j, encode_flash(MERGE_FLASH_FRAMES, true));
}

@compute @workgroup_size(256)
fn prepare(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: u32,
    @builtin(workgroup_id) wgid: vec3<u32>,
) {
    if (params.cell_size_mode == MERGE_CELL_MODE_GPU_ADAPTIVE) {
        if (lid == 0u) {
            wg_max_radius = 0.0;
        }
        workgroupBarrier();
    }

    let i = gid.x;
    var local_max_radius = 0.0;
    if (i < params.n) {
        let flash = flash_counter(absorbed(i));
        var next_flash = flash;
        if (flash > 0u) {
            next_flash = flash - 1u;
        }
        set_absorbed(i, next_flash << 1u);
        set_bucket_next(i, INVALID);
        let mass = masses[i];
        let radius = physical_radius_from_mass(mass);
        scratch[i] = vec4<f32>(positions[i].xyz, mass);
        scratch[SCRATCH_VEL_RADIUS_OFFSET + i] = vec4<f32>(velocities[i].xyz, radius);
        if (mass > params.min_mass) {
            local_max_radius = radius;
        }
    }

    if (params.cell_size_mode == MERGE_CELL_MODE_GPU_ADAPTIVE) {
        var<workgroup> shared: array<f32, 256>;
        shared[lid] = local_max_radius;
        workgroupBarrier();

        var stride = 128u;
        loop {
            if (lid < stride) {
                shared[lid] = max(shared[lid], shared[lid + stride]);
            }
            workgroupBarrier();
            if (stride == 1u) {
                break;
            }
            stride = stride / 2u;
        }

        if (lid == 0u) {
            scratch[SCRATCH_PARTIAL_RADIUS_OFFSET + wgid.x] =
                vec4<f32>(shared[0], 0.0, 0.0, 0.0);
        }
    }
}

@compute @workgroup_size(256)
fn finalize_cell_size(@builtin(local_invocation_id) lid: u32) {
    if (params.cell_size_mode != MERGE_CELL_MODE_GPU_ADAPTIVE || lid != 0u) {
        return;
    }

    var radius_cap = 0.0;
    for (var w = 0u; w < params.radius_partial_count; w++) {
        radius_cap = max(radius_cap, scratch[SCRATCH_PARTIAL_RADIUS_OFFSET + w].x);
    }
    if (radius_cap <= 0.0) {
        radius_cap = physical_radius_from_mass(params.min_mass);
    }
    let inv = radius_cap_to_inv_cell_size(radius_cap);
    scratch[SCRATCH_METADATA_INDEX] = vec4<f32>(radius_cap, inv, 0.0, 0.0);
}

@compute @workgroup_size(256)
fn clear_buckets(@builtin(global_invocation_id) gid: vec3<u32>) {
    let b = gid.x;
    if (b >= arrayLength(&bucket_heads)) {
        return;
    }
    atomicStore(&bucket_heads[b], INVALID);
}

@compute @workgroup_size(256)
fn init_owner(@builtin(global_invocation_id) gid: vec3<u32>) {
    let j = gid.x;
    if (j >= params.n) {
        return;
    }
    atomicStore(&merge_owner[j], params.n);
}

@compute @workgroup_size(256)
fn build_grid(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n || snap_mass(i) <= params.min_mass) {
        return;
    }
    let c = cell_coords(snap_pos(i));
    let b = hash_cell(c.x, c.y, c.z);
    var prev = atomicLoad(&bucket_heads[b]);
    loop {
        set_bucket_next(i, prev);
        let result = atomicCompareExchangeWeak(&bucket_heads[b], prev, i);
        if (result.exchanged) {
            break;
        }
        prev = result.old_value;
    }
}

@compute @workgroup_size(256)
fn find_owner(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n || snap_mass(i) <= params.min_mass) {
        return;
    }
    let c = cell_coords(snap_pos(i));
    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let b = hash_cell(c.x + dx, c.y + dy, c.z + dz);
                var j = atomicLoad(&bucket_heads[b]);
                while (j != INVALID) {
                    let j_next = bucket_next(j);
                    if (mergeable(i, j)) {
                        atomicMin(&merge_owner[j], i);
                    }
                    j = j_next;
                }
            }
        }
    }
}

@compute @workgroup_size(256)
fn apply_merge(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n || snap_mass(i) <= params.min_mass) {
        return;
    }
    let c = cell_coords(snap_pos(i));
    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let b = hash_cell(c.x + dx, c.y + dy, c.z + dz);
                var j = atomicLoad(&bucket_heads[b]);
                while (j != INVALID) {
                    let j_next = bucket_next(j);
                    if (atomicLoad(&merge_owner[j]) == i && !absorbed_this_pass(absorbed(j)) && mergeable(i, j)) {
                        absorb(i, j);
                    }
                    j = j_next;
                }
            }
        }
    }
}
