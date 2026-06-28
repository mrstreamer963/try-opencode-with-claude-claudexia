## 1. Rust core: data model

- [x] 1.1 Add `move_progress: f32` field to `Path` struct in `game-core/src/components.rs`
- [x] 1.2 Initialize `move_progress: 0.0` at every `Path` construction site (search the crate for `Path {` to find them; likely in `engine.rs` task-assignment code)
- [x] 1.3 Add `pub fn move_cost(tile: TileType) -> f32` to `game-core/src/world.rs` returning Grass=1.0, Sand=1.8, Water=`f32::INFINITY`

## 2. Rust core: movement logic

- [x] 2.1 Add `const BASE_MOVE_SPEED: f32 = 3.0;` to `game-core/src/engine.rs`
- [x] 2.2 Rewrite `move_colonists(world, dt)` in `engine.rs`: replace per-tick step-snapping with the rate-based progress + carry-the-remainder loop described in `design.md` § Decisions
- [x] 2.3 Preserve existing path-completion semantics: when `current_step >= steps.len()`, remove the `Path` component and trigger the same task transition (MovingToFood→Eating, MovingToBed→Sleeping, wander→Idle) as today
- [x] 2.4 Verify with `cargo check -p game-core` that the crate compiles

## 3. Rust core: snapshot

- [x] 3.1 Extend `ColonistState` in `game-core/src/events.rs` with `pub next_x: Option<i32>`, `pub next_y: Option<i32>`, `pub move_progress: f32`
- [x] 3.2 Update `emit_snapshot` in `engine.rs` to populate the new fields by looking up each colonist's optional `Path` (with `Path` → `Some(steps[current_step])` and the float progress; without `Path` → `None`/`None`/`0.0`)
- [x] 3.3 Rebuild the WASM artifact (run the project's existing wasm-pack/build script) and confirm `src/wasm/game_core` regenerates with the new field shape

## 4. TypeScript types

- [x] 4.1 Add `next_x: number | null`, `next_y: number | null`, `move_progress: number` to `ColonistState` in `src/types/events.ts`
- [x] 4.2 Run `tsc --noEmit` (or `npm run typecheck` if defined) and resolve any consumers that now miss the fields — `ColonistInfoPanel` should not need changes, but verify

## 5. Renderer: cadence split

- [x] 5.1 In `src/components/GameCanvas.vue`, refactor the Pixi ticker callback so that the terrain/buildings dirty-gate (`snap === lastSnapshotRef` early return) wraps ONLY `renderTerrain` and `renderBuildings`
- [x] 5.2 Ensure `renderEntities(snap.colonists)` and `renderBlueprint()` are called on every Pixi ticker frame whenever a snapshot exists, regardless of whether the reference has changed
- [x] 5.3 Keep the `watch(() => props.placementMode, …)` hook as-is — it now redundantly triggers `renderBlueprint`, but that's harmless; remove only if it causes lint warnings

## 6. Renderer: interpolation

- [x] 6.1 In `renderEntities`, before computing `cx`/`cy`, compute the interpolated float position: if `c.next_x != null && c.next_y != null` then `fx = c.x + (c.next_x - c.x) * c.move_progress`, `fy = c.y + (c.next_y - c.y) * c.move_progress`; otherwise `fx = c.x`, `fy = c.y`
- [x] 6.2 Use `fx`/`fy` (not `c.x`/`c.y`) when computing `cx = fx * TILE_SIZE + TILE_SIZE / 2` and `cy = fy * TILE_SIZE + TILE_SIZE / 2`
- [x] 6.3 Leave color, task indicators, and radius unchanged

## 7. Manual verification

- [ ] 7.1 Run `npm run dev` and confirm colonists glide between cells with no visible teleport
- [ ] 7.2 Place a path that crosses Sand and confirm the colonist is visibly slower on Sand than on Grass
- [ ] 7.3 Pause: colonists freeze mid-cell at the exact interpolated position; resume continues from there
- [ ] 7.4 Speed ×2 / ×3: motion is proportionally faster and remains smooth
- [ ] 7.5 Stationary colonists (idle / eating / sleeping) render exactly on the cell center, with no jitter
- [ ] 7.6 Task arrival: when a colonist reaches a Bed or BerryBush, it snaps to the final cell and the task transitions correctly (no residual sub-cell drift)
- [ ] 7.7 `ColonistInfoPanel`: position display still updates and is the integer "from" cell during transitions (expected)

## 8. Wrap-up

- [x] 8.1 Run any linters/formatters the project uses (`cargo fmt`, `npm run lint` if defined)
- [ ] 8.2 Commit changes with a message referencing the OpenSpec change name (`smooth-colonist-movement`)
