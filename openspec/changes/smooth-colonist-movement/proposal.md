## Why

Colonists currently teleport from cell to cell — every 50 ms simulation tick advances `Position` to the next path step instantly, and the renderer draws the colonist on the exact integer cell center. The discrete motion looks jerky and breaks the feel of a living colony. We want RimWorld-style smooth visuals while keeping the simulation simple and deterministic.

## What Changes

- Add a float `move_progress` (0.0..1.0) to the `Path` component representing progress toward the next cell.
- Add `next_x`, `next_y`, and `move_progress` fields to the `ColonistState` snapshot so the renderer can interpolate.
- Replace the tick-snapping logic in `move_colonists` with rate-based progress accumulation that carries the remainder across cell boundaries.
- Introduce per-tile `move_cost` (Grass 1.0, Sand 1.8, Water ∞) so terrain affects walking speed, mirroring RimWorld's `pathCost`.
- Split the renderer cadence: terrain/buildings remain dirty-gated on snapshot changes, but entities and the placement blueprint re-render every Pixi frame.
- Add lerp logic in `renderEntities` so a moving colonist draws at the interpolated float position between `(x, y)` and `(next_x, next_y)`.

Non-breaking for the simulation contract: pathfinding, reservations, occupancy checks, task assignment, and the `(x, y)` semantics in `ColonistInfoPanel` stay unchanged. The snapshot JSON gains fields but does not lose any.

## Capabilities

### New Capabilities
<!-- None — this change extends existing capabilities. -->

### Modified Capabilities
- `game-core`: `Colonist Needs & AI` movement behavior changes from per-tick cell-snapping to rate-based progress accumulation with per-tile cost; `Tick-Based Simulation` snapshot schema gains `next_x`/`next_y`/`move_progress` fields on `ColonistState`.
- `view-layer`: `PixiJS Canvas Rendering` entity rendering becomes per-frame and interpolates between cells; `Render optimization` dirty-gating is restricted to terrain and buildings.

## Impact

- **Rust core (`game-core/src/`):** `components.rs` (`Path`), `world.rs` (new `move_cost`), `engine.rs` (rewrite `move_colonists`, update `emit_snapshot`), `events.rs` (`ColonistState`).
- **TypeScript types (`src/types/events.ts`):** add three nullable/numeric fields to `ColonistState`.
- **Renderer (`src/components/GameCanvas.vue`):** split the render pipeline into per-snapshot and per-frame passes; add interpolation in `renderEntities`.
- **Worker (`src/worker/gameWorker.ts`):** no change — already passes `BASE_DT * speedMultiplier`, so pause/speed continue to work correctly.
- **Other Vue components (HUD, Toolbar, ColonistInfoPanel):** no change.
- **Build / dependencies:** no new dependencies.
