# Feature: パフォーマンス最適化

## 目的

10,000 体デフォルト設定で macOS（Metal）上 **8.6 fps / 116 ms/frame** となっており、インタラクティブな観察に不足している。  
星数削減・物理近似のスキップは行わず、**挙動を維持したまま**フレーム時間を短縮する。

## 計測サマリ

詳細: [measurements.md](measurements.md)

| 指標 | 現状 | 目標（Phase 1 後） |
|------|------|-------------------|
| Frame | 116 ms | ≤ 50 ms（20 fps 以上） |
| FPS | 8.6 | ≥ 20 |
| active bodies | 10,000 | 変更なし |
| merge 時の体感 | やや重い | 改善（Phase 2） |

Metal では pass 別 GPU 時間が取れないため、**Frame ms を主 KPI** とする。

## 計画概要

### Phase 1: 低リスク・即効性（挙動同一）
ディスパッチ最適化、シェーダ micro-opt、CPU オーバーヘッド削減。  
→ [phase1_quick_wins.md](phase1_quick_wins.md)

### Phase 2: Merge パス最適化（挙動同一）
merge 6 パスの GPU コスト削減。クラスタ時の悪化を緩和。  
→ [phase2_merge_optimization.md](phase2_merge_optimization.md)

### Phase 3: 描画最適化（見た目は近似可、物理不変）
184 万頂点 draw の負荷低減。  
→ [phase3_render_optimization.md](phase3_render_optimization.md)

### Phase 4: 計測基盤強化
Metal でも GPU 時間を把握できる手段と、自動ベンチマーク。  
→ [phase4_profiling.md](phase4_profiling.md)

### Phase 5: Merge 空間ハッシュの適応的セルサイズ（挙動同一）
小半径・密集配置（太陽系スケール）で merge 探索が O(N²) 化する問題を、セルサイズの動的決定で緩和。  
→ [phase5_merge_grid_adaptive.md](phase5_merge_grid_adaptive.md)

## スコープ外

- 星数（active_count）の削減
- Barnes-Hut / FMM 等の近似重力（物理挙動が変わる）
- gravity の全対全計算のスキップ

## 依存関係

```
計測基盤（済） → Phase 1 → Phase 2
                         ↘ Phase 3（並行可）
Phase 4（計測強化）は Phase 1 と並行可
```

## 受け入れ条件（Feature 全体）

- [ ] デフォルト設定（10k 体・同一 seed）で Frame ms が Phase 1 適用前比 **30% 以上改善**
- [ ] merge 密集シナリオで Merge 関連 pass の寄与が増えても Frame ms の増加が **Phase 1 前より小さい**
- [ ] 物理結果（同一 seed・同一 dt で N フレーム後の body 状態）が Phase 1/2 適用前後で一致
- [ ] 計測手順が [measurements.md](measurements.md) に文書化されている
