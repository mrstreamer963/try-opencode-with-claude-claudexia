## 1. Project Scaffolding

- [x] 1.1 Initialize Vite project with Vue 3 + TypeScript (`npm create vite@latest` with vue-ts template)
- [x] 1.2 Install frontend dependencies: `pixi.js@^8`, `vite-plugin-wasm`, `vite-plugin-top-level-await`
- [x] 1.3 Create Rust crate: `cargo init --lib game-core` with `Cargo.toml` configured for `cdylib` target, add deps: `bevy_ecs`, `wasm-bindgen`, `serde`, `serde_json`
- [x] 1.4 Configure Vite: add WASM plugin, configure worker entry point, verify dev server serves `.wasm` correctly
- [x] 1.5 Add `wasm-pack build` script and verify Rust → WASM compilation succeeds

## 2. Shared Types & Message Protocol

- [x] 2.1 Define TypeScript types for IncomingEvent union (PlaceBuilding, SetSpeed, Pause, Resume) and OutgoingEvent union (Ready, WorldSnapshot, BuildingPlaced, Error) in `src/types/events.ts`
- [x] 2.2 Define Rust-side `IncomingEvent` and `OutgoingEvent` enums with `serde(tag = "type")` in `game-core/src/events.rs`
- [x] 2.3 Define shared data structures: `TileType`, `ColonistState`, `BuildingState`, `WorldSnapshot` in both Rust (serde) and TypeScript

## 3. Game Core — World & Tiles

- [x] 3.1 Implement `World` resource: flat `Vec<TileType>` of 50×50, indexed by `y * WIDTH + x`, with helper methods `get_tile(x, y)`, `set_tile(x, y)`, `is_walkable(x, y)`
- [x] 3.2 Implement world generation: randomize tiles (Water/Sand/Grass) ensuring connected walkable area (flood fill validation)
- [x] 3.3 Expose WASM entry point: `init()` → creates ECS world with tile grid, returns serialized initial WorldSnapshot

## 4. Game Core — ECS Components & Entities

- [x] 4.1 Define ECS components: `Position { x, y }`, `Needs { food, sleep }`, `ColonistTag`, `BuildingType(Wall|Bed|BerryBush)`, `CurrentTask`, `Path(Vec<(i32,i32)>)`
- [x] 4.2 Implement colonist spawning system: create 3 colonists at random walkable positions with needs initialized to 1.0
- [x] 4.3 Implement building entity creation from PlaceBuilding events with tile validation (not water, not occupied)

## 5. Game Core — A* Pathfinding

- [x] 5.1 Implement A* pathfinding on the flat tile grid with binary heap, respecting `is_walkable` (water and walls block)
- [x] 5.2 Path targets should resolve to a walkable tile adjacent to the destination entity
- [x] 5.3 Handle unreachable targets: cancel task and return colonist to idle

## 6. Game Core — Needs & Task Systems

- [x] 6.1 Implement needs decay system: decrease Food and Sleep by `decay_rate * dt` each tick
- [x] 6.2 Implement task assignment system: idle colonists check lowest need → if below threshold, seek nearest BerryBush (food) or Bed (sleep)
- [x] 6.3 Implement eating behavior: colonist at BerryBush restores Food over time
- [x] 6.4 Implement sleeping behavior: colonist at Bed restores Sleep over time
- [x] 6.5 Implement idle wandering: colonists with no urgent needs walk to random nearby tile

## 7. Game Core — Tick & Event Processing

- [x] 7.1 Implement `tick(dt)` function: process incoming events → decay needs → assign tasks → move colonists along paths → emit OutgoingEvents
- [x] 7.2 Implement IncomingEvent queue: `push_event(json: &str)` parses and queues events for next tick
- [x] 7.3 Implement OutgoingEvent emission: `drain_events() -> String` returns JSON array of all events emitted during last tick
- [x] 7.4 Implement WorldSnapshot serialization: colonist states (position, needs, task), building positions/types, tile data (on first tick or change)

## 8. Worker Bridge — Worker Side

- [x] 8.1 Create worker entry file (`src/worker/gameWorker.ts`): import WASM module, call `init()`, post Ready event
- [x] 8.2 Implement game loop: `setInterval(50ms)` calling `tick(dt * speedMultiplier)`, post outgoing events to main thread
- [x] 8.3 Handle incoming messages: parse IncomingEvent, forward to WASM via `push_event()`, handle Pause/Resume/SetSpeed locally (stop/start interval, update multiplier)

## 9. Worker Bridge — Main Thread Side

- [x] 9.1 Create bridge module (`src/bridge/gameBridge.ts`): spawn worker, provide `send(event: IncomingEvent)` method
- [x] 9.2 Implement event listener registration: `on(type, callback)` / `off(type, callback)` for OutgoingEvent dispatch
- [x] 9.3 Provide Vue composable `useGameBridge()` with reactive state: `isReady`, `isPaused`, `speed`, `latestSnapshot`

## 10. View Layer — PixiJS Renderer

- [x] 10.1 Create PixiJS application component: initialize Application, mount canvas, set up 3 Container layers (terrain, blueprints, entities)
- [x] 10.2 Implement terrain renderer: on WorldSnapshot, draw 50×50 colored rectangles (Water=blue, Sand=yellow, Grass=green) on the terrain layer
- [x] 10.3 Implement entity renderer: draw colonist sprites (colored circles/shapes) and building sprites on the entity layer, update positions from WorldSnapshot
- [x] 10.4 Implement blueprint layer: show semi-transparent preview of selected building type at hovered tile during placement mode
- [x] 10.5 Implement `requestAnimationFrame` render loop: use latest WorldSnapshot, skip re-render if no new data

## 11. View Layer — Camera Controls

- [x] 11.1 Implement camera pan: click-drag on canvas translates the PixiJS stage position
- [x] 11.2 Implement camera zoom: mouse wheel adjusts stage scale with min/max clamp
- [x] 11.3 Implement world bounds clamping: restrict camera so it cannot scroll past the 50×50 tile grid

## 12. View Layer — Vue UI Components

- [x] 12.1 Create HUD component: Pause/Resume toggle button and speed buttons (1x, 2x, 3x) with active state highlight
- [x] 12.2 Create Toolbar component: Wall/Bed/BerryBush buttons, manage placement mode state, Escape to cancel
- [x] 12.3 Create ColonistInfoPanel component: displays name, Food/Sleep progress bars, current task, grid position; opens on colonist click, closes on empty-space click
- [x] 12.4 Wire click events: canvas click → detect colonist hit (PixiJS event) → open info panel; tile click in placement mode → send PlaceBuilding event

## 13. Integration & Polish

- [x] 13.1 Wire all components together: App.vue mounts PixiJS canvas + Vue HUD/Toolbar/InfoPanel overlays, connects to game bridge
- [x] 13.2 Handle error events from worker: display placement rejection errors as brief toast/notification
- [x] 13.3 End-to-end test: verify colonists spawn, needs decay, colonists seek food/bed, buildings can be placed, pause/speed work
- [x] 13.4 Add basic styling: position HUD/Toolbar/InfoPanel as overlays on the canvas with z-index layering
