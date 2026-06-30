# Phase 5: Merge 空間ハッシュの適応的セルサイズ

## 目的

星の大きさ・分布を小さくした場合（例: 中心星 1 M☉、10,000 体を 20 AU 以内）に merge パスが重くなる問題を解消する。  
空間ハッシュのセルサイズをシナリオに合わせて縮小し、**衝突判定の正しさを維持したまま** merge 探索コストを下げる。

## 背景・問題

### 現状のセルサイズ

```rust
// src/model/physics.rs
cell_size = max(2.0 * MERGE_MAX_RADIUS * merge_radius_factor, 0.01)
```

| 定数 / 設定 | 値 | 意味 |
|-------------|-----|------|
| `MERGE_MAX_RADIUS` | 0.25 AU | `STAR_MASS_MAX` 級の星を想定した固定半径上限 |
| `merge_radius_factor` | 20.0（デフォルト） | マージ距離 = `(r_i + r_j) × factor` |
| **デフォルト cell_size** | **10 AU** | 上式より |

### 太陽系スケールでのミスマッチ

| 量 | 典型値 |
|----|--------|
| ディスク星質量 | 0.002–0.02 M☉ |
| 物理半径 `r ∝ m^(1/3)` | ~0.0006–0.0013 AU |
| マージ距離 `(r_i + r_j) × 20` | ~0.03–0.05 AU |
| 現在の cell_size | **10 AU**（マージ距離の **~200 倍**） |

20 AU 領域は 10 AU セルだと **2–3 セル/軸** しかなく、10,000 体が **少数バケットの長い侵入リスト** に載る。  
`find_owner` / `apply_merge` は各体が **27 セル × リスト全走査** するため、実質 **O(N²)** に近づく。

### 重力との関係

空間ハッシュは **merge 専用**。重力は `gravity.wgsl` で全対全 O(N²) のまま。  
本 Phase は merge 高速化が主目的だが、太陽系シミュレーション全体の快適化には **別途 gravity 空間分割**（スコープ外）も必要。

---

## 方針候補の整理

| 案 | 内容 | 評価 |
|----|------|------|
| **A** 初期配置から決める | アップロード時に `radius_cap` を計算し固定 | ◎ コストゼロ。合体で星が大きくなると取りこぼしリスク（要安全上限） |
| **B** N フレームごと | 定期的に `radius_cap` を更新 | ○ C の更新間引きとして後から追加可能 |
| **C** 毎 prepare ごと | GPU で現フレームの最大半径を集計 | ◎ 正しさ・効果のバランスが最良 |

### 推奨

1. **案 A（安全な質量包絡）** を CPU 側で先に実装（shader 変更なしで効果確認可）
2. **案 C（AdaptivePerPrepare）** を GPU reduction で追加
3. 案 B は `AdaptivePeriodic { interval_frames }` として将来拡張

### 正しさ条件

セル辺長 ≥ 任意の活性体ペアの最大マージ距離:

```
d_merge ≤ (r_i + r_j) × merge_radius_factor ≤ 2 × r_cap × merge_radius_factor
```

`r_cap` は **現在または将来想定される最大物理半径** の安全上限とする。

---

## アーキテクチャ概要

```
┌─────────────────────────────────────────────────────────────┐
│  model/                                                      │
│  ┌──────────────────┐  ┌─────────────────────────────────┐  │
│  │ PhysicsSettings  │  │ MergeGridSizer                  │  │
│  │ - policy         │  │ - radius_from_mass              │  │
│  │ - cell_min       │  │ - initial_mass_envelope_cap     │  │
│  │ - safety factor  │  │ - cpu_inv_cell_size             │  │
│  └────────┬─────────┘  └──────────────┬──────────────────┘  │
└───────────┼───────────────────────────┼─────────────────────┘
            │                           │
┌───────────▼───────────────────────────▼─────────────────────┐
│  simulation/gpu/                                             │
│  ┌──────────────┐  ┌──────────────────┐  ┌───────────────┐  │
│  │ MergeParams  │  │ SimulationGpu    │  │ ComputeNode   │  │
│  │ (uniform)    │  │ Buffers          │  │ (pass 順序)   │  │
│  └──────┬───────┘  │ merge_scratch 拡張│  └───────┬───────┘  │
└─────────┼──────────┴──────────────────┴──────────┼──────────┘
          │                                        │
┌─────────▼────────────────────────────────────────▼───────────┐
│  assets/shaders/merge.wgsl                                   │
│  prepare → [finalize_cell_size] → clear → build → find → apply│
└──────────────────────────────────────────────────────────────┘
```

**制約**: WebGPU compute stage の storage buffer **最大 8 個**。新規 buffer は増やさず、既存 `merge_scratch` を拡張して metadata / partial reduction 領域を確保する。

---

## クラス（struct / module）ごとの責務

### 1. `PhysicsSettings` — `src/model/physics.rs`

**責務**: ユーザー設定・URL 永続化対象の物理パラメータ。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `softening` | `f32` | 既存 |
| `merge_radius_factor` | `f32` | 既存。マージ距離係数 |
| `merge_cell_policy` | `MergeCellSizePolicy` | セルサイズ決定方式 |
| `merge_cell_min_size` | `f32` | セルサイズ下限（AU） |
| `merge_cell_radius_safety` | `f32` | 半径上限への安全係数（≥ 1.0） |

**関数インターフェース**:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeCellSizePolicy {
    /// 現行互換: MERGE_MAX_RADIUS 固定
    ConservativeFixed,
    /// 初期質量分布から将来最大半径の安全上限を計算（案 A）
    InitialMassEnvelope,
    /// 各 merge_prepare ごとに GPU で最大半径を集計（案 C）
    AdaptivePerPrepare,
}

impl PhysicsSettings {
    pub fn softening_sq(&self) -> f32;

    /// 半径上限から cell size を計算（純粋関数）。
    pub fn merge_cell_size_from_radius_cap(&self, radius_cap: f32) -> f32;

    /// 半径上限から inv_cell_size を計算（純粋関数）。
    pub fn merge_inv_cell_size_from_radius_cap(&self, radius_cap: f32) -> f32;

    /// 現行互換の固定 inv_cell_size。
    pub fn conservative_merge_inv_cell_size(&self) -> f32;

    pub fn clamped(self) -> Self;
}
```

**cell size 計算式**:

```rust
fn merge_cell_size_from_radius_cap(&self, radius_cap: f32) -> f32 {
    let safe_radius = radius_cap * self.merge_cell_radius_safety;
    (2.0 * safe_radius * self.merge_radius_factor)
        .max(self.merge_cell_min_size)
}
```

---

### 2. `MergeGridSizer` — `src/model/merge_grid.rs`（新規）

**責務**: CPU 側で決定可能な merge cell size を計算する純粋ロジック（Bevy / GPU 非依存）。

**関数インターフェース**:

```rust
pub struct MergeGridSizer;

impl MergeGridSizer {
    /// 質量から物理半径を計算: `SUN_RADIUS_AU * mass^(1/3)`。
    pub fn radius_from_mass(mass: f32) -> f32;

    /// 現行互換の保守的 radius cap（= MERGE_MAX_RADIUS）。
    pub fn conservative_radius_cap() -> f32;

    /// 案 A: 初期質量分布から将来最大半径の安全上限。
    /// 質量保存を前提に `total_active_mass^(1/3)` を使用。
    pub fn initial_mass_envelope_radius_cap(masses: &[f32], active_count: usize) -> f32;

    /// policy に応じて CPU 側 inv_cell_size を返す。
    /// AdaptivePerPrepare の場合は GPU 側計算のため None。
    pub fn cpu_inv_cell_size(
        physics: &PhysicsSettings,
        masses: &[f32],
        active_count: usize,
    ) -> Option<f32>;
}
```

**案 A の安全上限**:

```rust
// 全活性体が 1 体に合体しても cell が小さすぎないよう total_mass から算出
let total_mass: f32 = masses[..active_count]
    .iter()
    .filter(|&&m| m > MIN_MASS)
    .sum();
let radius_cap = Self::radius_from_mass(total_mass);
```

---

### 3. `MergeParams` — `src/simulation/gpu/params.rs`

**責務**: merge compute shader へ渡す uniform。cell size policy と adaptive 用パラメータを含む。

**フィールド追加**:

```rust
#[derive(Clone, Copy, ShaderType, PartialEq)]
pub struct MergeParams {
    pub n: u32,
    pub merge_radius_factor: f32,
    pub inv_cell_size: f32,           // Fixed / InitialMassEnvelope モードで使用
    pub min_mass: f32,

    pub cell_size_mode: u32,          // 0 = Fixed, 1 = GpuAdaptive
    pub radius_partial_count: u32,    // workgroup 数
    pub merge_cell_min_size: f32,
    pub merge_cell_radius_safety: f32,
}

pub const MERGE_CELL_MODE_FIXED: u32 = 0;
pub const MERGE_CELL_MODE_GPU_ADAPTIVE: u32 = 1;
```

**関数インターフェース**:

```rust
impl MergeParams {
    pub fn from_settings(
        settings: &SimulationSettings,
        cpu_inv_cell_size: Option<f32>,
    ) -> Self;
}
```

---

### 4. `SimulationGpuBuffers` — `src/simulation/gpu/buffers.rs`

**責務**: GPU 常駐バッファ。`merge_scratch` を拡張して adaptive reduction 用領域を確保。

**定数**:

```rust
pub const MERGE_SCRATCH_BODY_OFFSET: usize = 0;
pub const MERGE_SCRATCH_VEL_RADIUS_OFFSET: usize = BODY_COUNT;
pub const MERGE_SCRATCH_METADATA_INDEX: usize = BODY_COUNT * 2;
pub const MERGE_SCRATCH_PARTIAL_RADIUS_OFFSET: usize = BODY_COUNT * 2 + 1;

pub const MAX_MERGE_WORKGROUPS: usize =
    BODY_COUNT.div_ceil(WORKGROUP_SIZE as usize);

pub const MERGE_SCRATCH_LEN: usize =
    BODY_COUNT * 2 + 1 + MAX_MERGE_WORKGROUPS;
```

**layout**:

```
[0 .. BODY_COUNT)                         pos.xyz + mass
[BODY_COUNT .. BODY_COUNT*2)              vel.xyz + radius
[BODY_COUNT*2]                            metadata (radius_cap, inv_cell_size)
[BODY_COUNT*2 + 1 .. + 1 + workgroups)   per-workgroup radius partials
```

**変更**: `merge_scratch` の初期化サイズを `MERGE_SCRATCH_LEN` に拡張。

---

### 5. `SimulationComputePipelines` — `src/simulation/gpu/pipelines.rs`

**責務**: merge adaptive 用 compute entry point を管理。

**追加フィールド**:

```rust
pub struct SimulationComputePipelines {
    // ... 既存 ...
    pub merge_prepare: CachedComputePipelineId,
    pub merge_finalize_cell_size: CachedComputePipelineId,  // 新規
    pub merge_clear_buckets: CachedComputePipelineId,
    // ...
}
```

---

### 6. `SimulationComputeNode` — `src/simulation/gpu/node.rs`

**責務**: 各 frame / merge iteration の compute pass 順序を決定。

**変更後の merge ループ**:

```
for _ in 0..MERGE_ITERATIONS_PER_FRAME:
    merge_prepare
    if cell_size_mode == GPU_ADAPTIVE:
        merge_finalize_cell_size    // 1 workgroup dispatch
    merge_clear_buckets
    merge_init_owner
    merge_build_grid
    merge_find_owner + merge_apply
```

**関数インターフェース（分割案）**:

```rust
fn run_merge_prepare(
    render_context: &mut RenderContext,
    diagnostics: &impl RecordDiagnostics,
    bind_groups: &SimulationComputeBindGroups,
    pipeline: &ComputePipeline,
    workgroups: u32,
);

fn run_merge_finalize_cell_size(
    render_context: &mut RenderContext,
    diagnostics: &impl RecordDiagnostics,
    bind_groups: &SimulationComputeBindGroups,
    pipeline: &ComputePipeline,
);

fn run_merge_iteration(/* 既存シグネチャ */);
```

---

### 7. `merge.wgsl` — `assets/shaders/merge.wgsl`

**責務**: GPU 側 merge 全処理。adaptive 時は prepare 内で partial max、finalize で global max。

**Params 拡張**:

```wgsl
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
```

**ヘルパー関数**:

```wgsl
fn physical_radius_from_mass(mass: f32) -> f32;

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
```

**entry point**:

| エントリ | 責務 |
|----------|------|
| `prepare` | snapshot、merge flash 更新、workgroup 内 max radius → partial 配列 |
| `finalize_cell_size` | partial 配列を reduce → metadata に `radius_cap`, `inv_cell_size` 書込 |
| `build_grid` | `current_inv_cell_size()` でセル座標計算 |
| `find_owner` / `apply_merge` | 変更なし（`cell_coords` 経由で adaptive cell を使用） |

---

## 案 A / B / C の実装マッピング

| 案 | `MergeCellSizePolicy` | 実行タイミング | inv_cell_size 決定場所 |
|----|----------------------|----------------|------------------------|
| A | `InitialMassEnvelope` | シミュレーション開始 / Restart 時 | CPU (`MergeGridSizer`) |
| B | `AdaptivePeriodic { interval_frames }`（将来） | N フレームごと | GPU（C と同じ pass、実行頻度のみ間引き） |
| C | `AdaptivePerPrepare` | 各 merge iteration の prepare 後 | GPU (`finalize_cell_size`) |
| 現行互換 | `ConservativeFixed` | 変更なし | CPU（`MERGE_MAX_RADIUS` 固定） |

---

## 実装フェーズ

### Step 1: モデル層（shader 変更なし）

- [ ] `MergeCellSizePolicy` を `PhysicsSettings` に追加
- [ ] `merge_cell_min_size`, `merge_cell_radius_safety` 追加
- [ ] `MergeGridSizer` 新規 module
- [ ] `PhysicsSettings::merge_cell_size_from_radius_cap` 等を実装
- [ ] `ConservativeFixed` をデフォルトにし現行挙動を維持

### Step 2: 案 A（InitialMassEnvelope）

- [ ] Restart / upload 時に `MergeGridSizer::initial_mass_envelope_radius_cap` を呼ぶ
- [ ] 結果を `MergeParams.inv_cell_size` に反映
- [ ] 太陽系プリセット（1 M☉ + 10k 小質量 / 20 AU）で merge ベンチ

### Step 3: GPU adaptive（案 C）

- [ ] `merge_scratch` layout 拡張 + 定数注入（`shaders.rs`）
- [ ] `MergeParams` 拡張 + bind group 更新
- [ ] `merge.wgsl`: `prepare` に partial max 追加
- [ ] `merge.wgsl`: `finalize_cell_size` 新規 entry
- [ ] `merge.wgsl`: `current_inv_cell_size()` 導入
- [ ] `SimulationComputePipelines` / `node.rs` に finalize pass 追加
- [ ] profiling pass 名 `merge_finalize_cell_size` 追加

### Step 4: UI / URL（任意）

- [ ] Physics パネルに policy 選択（Advanced）
- [ ] URL エンコードに `merge_cell_policy` 追加

---

## 計測・受け入れ条件

### ベンチマークシナリオ

| シナリオ | 設定 |
|----------|------|
| 太陽系型 | 中心 1 M☉、10,000 体、disk R_outer ≤ 20 AU、典型質量 0.002–0.02 M☉ |
| 超大質量型 | `STAR_MASS_MAX` 級の星を含む（`ConservativeFixed` 互換確認） |
| merge 密集型 | Phase 2 計画の merge 密集シナリオ |

### 受け入れ条件

- [ ] `ConservativeFixed` で現行と物理結果が一致（同一 seed / checksum）
- [ ] `InitialMassEnvelope` で太陽系型シナリオの Frame ms が改善
- [ ] `AdaptivePerPrepare` で合体後に星が大きくなっても merge 取りこぼしがない
- [ ] cell_size が `merge_cell_min_size` 未満にならない
- [ ] storage buffer 数が 8 個を超えない
- [ ] `GRAVITIUM_BENCH=1` で A/B 比較結果を [measurements.md](measurements.md) に追記

---

## スコープ外

- 重力 `gravity.wgsl` の空間分割（別 Feature）
- `merge_radius_factor` と cell size の UI 分離（将来検討）
- 案 B（Periodic）— Step 3 完了後に必要なら追加

---

## 関連ドキュメント

- [phase2_merge_optimization.md](phase2_merge_optimization.md) — merge pass 融合・bucket チューニング
- [measurements.md](measurements.md) — 計測手順
- [../wasm_web/phase2_gpu_simulation.md](../wasm_web/phase2_gpu_simulation.md) — merge 空間ハッシュ仕様・採用しなかった案
- [../simulation_control/phase2_physics_params.md](../simulation_control/phase2_physics_params.md) — `PhysicsSettings` / `merge_inv_cell_size`
