# Smooth Colonist Movement (RimWorld-style)

**Date:** 2026-06-28
**Status:** Approved design, ready for implementation plan

## Problem

Colonists currently teleport from cell to cell. Every simulation tick (50 ms) the engine advances `Position` to the next path step instantly, and the renderer draws the colonist exactly at the integer cell center. The result is jerky, discrete motion.

## Approach: RimWorld-style cell-based simulation + render-time interpolation

The simulation stays **integer-cell-based** (A* pathfinding, collisions, reservations, task targets — all keep their existing simple, deterministic semantics). What changes is that each moving colonist also carries a **float progress** in `[0, 1]` for the transition from its current cell to its next cell. The world snapshot exposes this progress along with the next cell, and PixiJS linearly interpolates the draw position every frame.

This mirrors RimWorld's `Pawn_PathFollower`, which tracks `lastCell` / `nextCell` / `nextCellCostLeft` and lerps in the render layer while the sim logic remains tile-locked.

### Why this approach

- **Logic stays simple.** Pathfinding, occupancy checks, reservation logic, and task targets remain integer-cell based. No changes needed there.
- **Pause and speed work for free.** The worker already passes `BASE_DT * speedMultiplier` to `tick(dt)`, and stops calling `tick` when paused. Progress accumulates accordingly — no separate animation system to coordinate.
- **Per-terrain speed is natural.** Different tile types have different `move_cost`; progress advances at `dt * BASE_MOVE_SPEED / move_cost(target_tile)`. Mirrors RimWorld's per-terrain `pathCost`.
- **Minimally invasive.** Three new fields on `ColonistState`, one new field on `Path`, ~30 LOC changed across Rust + TS.

## Decisions

| Question | Decision |
|---|---|
| Where smooth motion lives | Rust owns float `move_progress`; renderer just lerps. (Hybrid / RimWorld-style.) |
| Per-tile speed | Yes — `move_cost` depends on tile type (Grass = 1.0, Sand = 1.8, Water = ∞). |
| Base tempo | ~3 cells/sec on grass (~330 ms/cell). |
| Facing | None for v1 — colonists render as symmetric circles, no rotation. |

## Changes

### 1. Rust core (`game-core/src/`)

**`components.rs` — extend `Path`:**

```rust
pub struct Path {
    pub steps: Vec<(i32, i32)>,
    pub current_step: usize,
    pub move_progress: f32,   // 0.0..1.0 toward steps[current_step]
}
```

When `Path` component is absent → colonist is stationary, no interpolation.

**`world.rs` — per-tile movement cost:**

```rust
pub fn move_cost(tile: TileType) -> f32 {
    match tile {
        TileType::Grass => 1.0,
        TileType::Sand  => 1.8,
        TileType::Water => f32::INFINITY,
    }
}
```

`Water = INFINITY` is defensive — pathfinding already won't path onto water, but this guarantees `move_progress` can never increment onto an unwalkable tile if something slips through.

**`engine.rs` — constant:**

```rust
const BASE_MOVE_SPEED: f32 = 3.0;  // cells/sec on grass baseline
```

**`engine.rs` — rewrite `move_colonists(dt)`:**

For each entity with `Path`:

1. If `current_step >= steps.len()` → mark completed (handled as today).
2. Otherwise:
   - Let `(nx, ny) = steps[current_step]`.
   - Let `cost = move_cost(tile_at(nx, ny))`.
   - `move_progress += dt * BASE_MOVE_SPEED / cost`.
   - **While** `move_progress >= 1.0` **and** there are still steps:
     - `Position = (nx, ny)`; `current_step += 1`; `move_progress -= 1.0`.
     - If `current_step >= steps.len()` → mark completed, break.
     - Otherwise refresh `(nx, ny)` and `cost` for the new step and continue.

The "carry the remainder" loop handles cases where a single tick crosses more than one cell (e.g. fast game speed × cheap terrain × long `dt`) without losing motion or stalling on cell boundaries.

After the loop, completed entities go through the same task-transition logic as today (remove `Path`, advance `CurrentTask` from `MovingToFood` → `Eating`, etc.).

**`events.rs` — extend `ColonistState`:**

```rust
pub struct ColonistState {
    pub id: u32,
    pub name: String,
    pub x: i32,                    // "from" cell — current logical position
    pub y: i32,
    pub next_x: Option<i32>,       // None if not moving
    pub next_y: Option<i32>,
    pub move_progress: f32,        // 0.0 if not moving
    pub food: f32,
    pub sleep: f32,
    pub task: ColonistTask,
}
```

`x`/`y` remain the integer "from" cell during a transition — analogous to RimWorld's `lastCell`. `ColonistInfoPanel` continues to show this as the canonical position.

**`engine.rs` — `emit_snapshot`:** when serializing a colonist, look up its optional `Path`:
- With `Path`: `next_x/next_y = Some(steps[current_step])`, `move_progress = path.move_progress`.
- Without `Path`: `next_x/next_y = None`, `move_progress = 0.0`.

### 2. TypeScript types (`src/types/events.ts`)

Add to `ColonistState`:

```ts
next_x: number | null
next_y: number | null
move_progress: number
```

### 3. Renderer (`src/components/GameCanvas.vue`)

**Split rendering into two cadences:**

Today, `render()` early-returns when the snapshot reference hasn't changed. That's correct for terrain/buildings (they only change on snapshot updates) but kills smoothness for colonists (no redraw between the 50 ms snapshots). Split the work:

- **Per-snapshot (dirty-gated):** `renderTerrain`, `renderBuildings`.
- **Per-frame (every Pixi ticker tick):** `renderEntities`, `renderBlueprint`.

Concretely, the ticker callback always invokes `renderEntities(snap.colonists)` and `renderBlueprint()` if a snapshot exists, and only invokes the terrain/building passes when `snap !== lastSnapshotRef`.

**Interpolate in `renderEntities`:**

```ts
const hasTarget = c.next_x != null && c.next_y != null
const fx = hasTarget ? c.x + (c.next_x! - c.x) * c.move_progress : c.x
const fy = hasTarget ? c.y + (c.next_y! - c.y) * c.move_progress : c.y
const cx = fx * TILE_SIZE + TILE_SIZE / 2
const cy = fy * TILE_SIZE + TILE_SIZE / 2
```

The rest of the per-colonist render code (color, task indicators) is unchanged.

**Note on smoothness:** between snapshots (50 ms) `move_progress` is constant — Rust only updates it on a tick. At 3 cells/sec, one tick = ~15% of a cell, which the eye reads as smooth motion without any client-side extrapolation. No `performance.now()`-based prediction needed for v1.

## What does NOT change

- A* pathfinding and `find_path_adjacent` — unchanged, integer-cell.
- Reservation / occupancy / wall collision — checked against integer `Position`.
- `assign_tasks`, `process_tasks`, needs decay — unchanged.
- Worker (`gameWorker.ts`) — already passes `BASE_DT * speedMultiplier`; pause stops the interval. No change.
- Vue components outside `GameCanvas.vue` (HUD, Toolbar, ColonistInfoPanel) — unchanged.
- Building placement, tile rendering — unchanged.
- Snapshot transport (JSON over `postMessage`) — unchanged shape, just extra fields.

## Testing

Manual verification via `npm run dev`:

1. **Smoothness:** colonists glide between cells, no visible teleporting.
2. **Per-tile speed:** colonists walking on sand are visibly slower than on grass.
3. **Pause:** clicking pause freezes colonists mid-cell instantly; resume continues from the same fractional position.
4. **Speed multiplier:** ×2 / ×3 makes them move proportionally faster, still smooth.
5. **Stationary colonists** (`next_x` null): rendered exactly on the cell center, no jitter.
6. **Task transitions:** when arriving at a bed/bush, the colonist snaps to the final cell and starts eating/sleeping (no residual sub-cell drift).
7. **`ColonistInfoPanel`:** position display matches the integer cell the colonist is "from" during a transition (acceptable — same as today; reflects logical position).

No automated tests added in this pass — the existing project has none, and visual motion is the success criterion.

## Out of scope

- Facing direction / sprite rotation (revisit when directional sprites are added).
- Diagonal-vs-orthogonal cost difference (current A* already handles cardinals only — if diagonals are added later, `move_cost` will need a √2 multiplier for diagonal steps).
- Client-side extrapolation between snapshots (not needed at this tick rate).
- Smooth start/stop easing (linear is fine and matches RimWorld).
- Per-pawn `Moving` stat / status effects on speed (no such system exists yet).
