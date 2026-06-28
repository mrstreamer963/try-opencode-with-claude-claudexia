## Context

The Rust game core ticks at 20 Hz (50 ms intervals; `BASE_DT = 0.05s`) via `gameWorker.ts`, calling `tick(dt)` where `dt = BASE_DT * speedMultiplier`. The current `move_colonists(dt)` system in `game-core/src/engine.rs` advances a colonist's `Position` to `Path::steps[current_step]` once per tick and increments the step index. The renderer in `src/components/GameCanvas.vue` reads the snapshot's `c.x`/`c.y` (always integers) and draws at the cell center. Because both the simulation step and the renderer use integer cells, motion is visibly discrete — colonists "teleport" 16 px every 50 ms.

The reference target for feel is RimWorld, whose `Pawn_PathFollower` keeps simulation tile-based but tracks `lastCell` / `nextCell` / `nextCellCostLeft` and lerps the draw position. We want the same separation here: integer-cell sim for determinism and clean pathfinding/reservation logic, float-per-frame interpolation for visuals.

The codebase already gives us pause/speed support nearly for free: pausing stops the worker interval, so `move_progress` cannot advance; speed multiplies `dt`, so `move_progress` advances proportionally faster. No new animation loop or wall-clock tracking is needed.

## Goals / Non-Goals

**Goals:**
- Smooth, continuous visual motion between adjacent cells with no perceptible jumps at 20 Hz tick rate.
- Per-terrain walking speed: Sand visibly slower than Grass, Water impassable.
- Pause and game-speed controls keep working unchanged.
- Logical state (`Position`, pathfinding, reservations, task targets) stays integer-cell.
- Minimally invasive: three new snapshot fields, one new component field, ~30 LOC changed.

**Non-Goals:**
- Facing direction / sprite rotation (colonists remain symmetric circles).
- Diagonal-vs-orthogonal cost differences (the current A* is 4-connected; revisit if it gains diagonals).
- Client-side extrapolation between snapshots using `performance.now()`.
- Easing / acceleration curves — linear progress matches RimWorld and looks fine.
- Per-pawn modifiers (no Moving stat exists yet).
- Automated tests — the project has none today; visual verification is the success criterion.

## Decisions

### Where smooth motion lives: Rust owns `move_progress`, renderer just lerps

**Alternatives considered:**
1. **Render-side interpolation only** — keep Rust unchanged, have the renderer lerp between previous and current snapshot positions over the tick interval. Smallest Rust change but couples render smoothness to snapshot history, complicates pause/speed (renderer would need to know game speed), and makes per-terrain speed awkward.
2. **Float positions in Rust** — make `Position` itself `(f32, f32)`. Maximally "correct" but invasive: pathfinding, reservation, occupancy, task-target equality checks would all need to switch from `==` to "close enough" semantics.
3. **Hybrid (chosen)** — keep `Position` integer, add `move_progress: f32` to `Path`. Renderer lerps using `(x, y) → (next_x, next_y)` with `move_progress` from the snapshot.

The hybrid mirrors RimWorld exactly, requires no changes to pathfinding/reservation/occupancy, and lets pause/speed work via the existing `dt`-on-tick mechanism. The cost is one extra field in `Path`, three extra fields in `ColonistState`.

### Carry the remainder across cell boundaries

When `move_progress` reaches 1.0 mid-tick, we advance `current_step`, subtract 1.0 from `move_progress`, and re-check whether the remainder crosses the next cell too (looping until either `move_progress < 1.0` or the path ends). This matters when a tick covers multiple cells — e.g. at 3× game speed on Grass (`dt = 0.15s`, base 3 cells/sec → 0.45 cells/tick at most, so usually one boundary per tick), but more importantly it prevents "stalling" on cell boundaries: if we just clamped to 1.0 and reset on the next tick, a colonist would lose progress every cross-boundary tick. The loop also handles future scenarios with cheaper terrain or higher speeds without re-architecting.

### Per-tile `move_cost` in `world.rs`

A small pure function `move_cost(tile: TileType) -> f32` returning `Grass=1.0`, `Sand=1.8`, `Water=f32::INFINITY`. The Sand multiplier is a feel choice (RimWorld's soil/sand ratios are roughly in this range). `Water = INFINITY` is defensive — pathfinding already won't path onto water, but if anything ever slips through, dividing by infinity makes progress zero, so the colonist freezes rather than teleporting onto an impassable tile.

### Snapshot uses the "from cell" semantics

`(x, y)` in `ColonistState` continues to mean the integer cell the colonist is currently in (or just left). `(next_x, next_y)` is the cell being entered, and `move_progress` ∈ [0, 1] is the fraction toward it. This matches RimWorld's `lastCell`/`nextCell` naming and means existing UI like `ColonistInfoPanel` keeps working unchanged — the position it displays is still the logical cell.

### Renderer cadence split

Currently `render()` short-circuits when the snapshot reference hasn't changed. That's correct for terrain and buildings (static between snapshots) but kills smoothness for entities. We split the work:

- **Per-snapshot (dirty-gated):** `renderTerrain`, `renderBuildings`.
- **Per-frame (every Pixi ticker tick):** `renderEntities`, `renderBlueprint` (blueprint also benefits — hover position can update between snapshots).

Per-frame entity drawing is cheap — Pixi's `Graphics` with a few circles per colonist at 50×50 grid scale.

### No wall-clock client-side extrapolation in v1

Between snapshots (50 ms), `move_progress` is constant on the client because Rust only updates it on a tick. At 3 cells/sec, that means each rendered frame within the 50 ms window shows the same interpolated position. Rendered at 60 fps that is 3 identical frames before the value steps. Visually this still reads as smooth motion because the *between-snapshot* progress steps (~15% of a cell each) are small. If, later, slower base speeds or higher tick intervals make the steps visible, we can add a `performance.now()`-based extrapolation `move_progress_visible = move_progress + (now - snapshot_time) * speed` clamped to 1.0. Out of scope now.

## Risks / Trade-offs

- **Snapshot size grows by 3 fields × N colonists.** With 3 colonists this is trivial; even at 100 colonists it's ~kilobytes per snapshot. → No mitigation needed.
- **`ColonistInfoPanel` shows the "from" cell during a transition.** While moving, the displayed position briefly differs from the visually rendered position. → Acceptable: the displayed cell is the logical, reservation-true position. Note in the docs that visual ≠ logical during transitions.
- **Per-frame entity draw could mask future perf issues.** Today there are 3 colonists; with hundreds, naive `Graphics.removeChildren()`+rebuild per frame becomes a hotspot. → Out of scope; revisit when entity count grows, possibly with Pixi `Sprite` reuse.
- **`move_progress` could overshoot if `dt` is huge** (e.g. tab backgrounded, then resumed). The carry-the-remainder loop handles this correctly — even crossing many cells in one tick still lands at integer cells. → Fine.
- **Diagonal pathfinding (future)** would need a √2 multiplier for diagonal `move_cost`. → Tracked in Out of Scope; one-line change when diagonals land.
- **Numerical drift on `move_progress -= 1.0`.** f32 subtraction can leave epsilon residues but the next iteration's comparison `>= 1.0` is correct, and we never use absolute equality. → Not a real issue.
