## Why

Build an idle colony simulation game (RimWorld-like) that runs entirely in the browser. The game uses a Rust ECS engine compiled to WASM for performant simulation inside a WebWorker, with a Vue 3 + PixiJS 8 frontend for rendering and UI. This architecture enables complex colony AI and pathfinding at near-native speed while keeping the game instantly accessible via any modern browser.

## What Changes

- Add a Rust-based ECS game core (bevy_ecs) compiled to WASM, running simulation logic (tile world, colonist AI, needs system, A* pathfinding, building placement) inside a WebWorker
- Add a Vue 3 + TypeScript + PixiJS 8 view layer with tile-based rendering (3 layers: terrain, blueprints, entities), camera controls (pan/zoom), HUD (pause/speed), toolbar (Wall/Bed/BerryBush), and colonist info panel
- Add a Worker Bridge layer using `postMessage` for host ↔ worker communication, with a 50ms game loop in the worker and `requestAnimationFrame`-driven rendering on the main thread
- Deliver v1 gameplay: 50x50 tile map (water/sand/grass), 3 colonists with Food/Sleep needs, automatic task assignment, no-resource building, and game speed controls with pause

## Capabilities

### New Capabilities
- `game-core`: Rust ECS engine — tile world (50x50), colonist entities with needs (Food/Sleep), building entities (Bed, BerryBush), A* pathfinding, automatic task scheduling, tick-based simulation via `wasm-bindgen`
- `view-layer`: Vue 3 + PixiJS 8 renderer — 3-layer tile rendering (terrain, blueprints, entities), camera pan/zoom, HUD (pause/speed buttons), building toolbar (Wall/Bed/BerryBush), colonist info panel (needs, task, position)
- `worker-bridge`: WebWorker communication bridge — `postMessage` event protocol (`IncomingEvent`/`OutgoingEvent`), 50ms game loop in worker, RAF-based render sync, WASM initialization via `vite-plugin-wasm`

### Modified Capabilities
<!-- None — greenfield project -->

## Impact

- **New Rust crate**: `game-core/` with `bevy_ecs`, `wasm-bindgen`, `serde`, `serde_json` dependencies; compiled to `wasm32-unknown-unknown`
- **New Vite project**: `web/` (or root-level) with Vue 3, PixiJS 8, TypeScript, `vite-plugin-wasm`
- **Build tooling**: Requires `wasm-pack` or `cargo` targeting WASM; Vite dev server must serve `.wasm` files from the worker
- **Browser requirements**: WebWorker support, WASM support (all modern browsers)
- **No backend**: Entirely client-side; no server APIs or databases
