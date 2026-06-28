## Context

Greenfield browser game project — no existing codebase. The goal is a RimWorld-inspired idle colony simulation running entirely client-side. The architecture splits into three layers: a Rust ECS simulation compiled to WASM (running in a WebWorker for off-main-thread performance), a Vue 3 + PixiJS 8 view layer on the main thread, and a postMessage-based bridge connecting them.

Key constraints:
- WASM must run inside a WebWorker (cannot block UI)
- Game state lives exclusively in the Rust ECS; the JS side holds only a render-ready snapshot
- PixiJS 8 handles canvas rendering; Vue handles DOM-based UI (HUD, panels)
- No backend — fully offline-capable

## Goals / Non-Goals

**Goals:**
- Deliver a playable v1: 50×50 tile world, 3 colonists with Food/Sleep needs, automatic task AI, building placement (Wall/Bed/BerryBush), pause/speed controls
- Clean separation of simulation (Rust/WASM) and presentation (Vue/PixiJS) via a message protocol
- Deterministic, tick-based simulation that can be paused and speed-adjusted
- Simple, extensible ECS architecture for adding more entities/components later

**Non-Goals:**
- Multiplayer or networked play
- Save/load persistence (future work)
- Procedural world generation beyond initial random placement
- Sound or music
- Mobile touch controls
- Resource costs for building (v1 builds for free)
- Complex colonist personality/mood systems

## Decisions

### D1: Rust + bevy_ecs for simulation (not pure JS)

**Choice**: Use Rust with `bevy_ecs` (standalone, not full Bevy engine), compiled to `wasm32-unknown-unknown` via `wasm-pack`.

**Rationale**: A* pathfinding on a 50×50 grid with multiple colonists and tick-based need decay benefits from Rust's performance. `bevy_ecs` provides a mature, ergonomic ECS without pulling in Bevy's renderer/windowing.

**Alternatives considered**:
- *Pure TypeScript ECS (bitecs/miniplex)*: Simpler build pipeline, but slower pathfinding and no benefit from Rust's type safety for simulation logic.
- *Full Bevy engine*: Includes its own renderer — conflicts with PixiJS and adds massive WASM bundle size.

### D2: WebWorker isolation for WASM

**Choice**: The WASM module runs exclusively inside a dedicated WebWorker. Main thread never calls WASM directly.

**Rationale**: `tick()` can take variable time depending on pathfinding load. Running on main thread would cause dropped frames. The worker also enables future parallelism.

**Alternatives considered**:
- *Main thread WASM*: Simpler architecture but risks UI jank.
- *SharedArrayBuffer*: Better latency but complex synchronization and requires COOP/COEP headers.

### D3: Message protocol (IncomingEvent / OutgoingEvent)

**Choice**: Typed JSON messages over `postMessage`. The worker accepts `IncomingEvent` variants (PlaceBuilding, SetSpeed, SelectColonist, etc.) and emits `OutgoingEvent` variants (WorldState snapshot, ColonistUpdate, etc.).

**Rationale**: Decouples simulation from rendering. Either side can be replaced independently. JSON is debuggable; `serde_json` makes Rust ↔ JS serialization trivial.

**Alternatives considered**:
- *Binary protocol (bincode/MessagePack)*: Better perf for large state, but premature for a 50×50 world — JSON is fast enough and much easier to debug.
- *Shared memory*: Avoids serialization cost but requires manual memory layout coordination.

### D4: PixiJS 8 for canvas rendering, Vue 3 for DOM UI

**Choice**: PixiJS renders tiles/sprites on a WebGL canvas. Vue manages HUD, toolbar, and info panels as DOM overlays.

**Rationale**: PixiJS excels at 2D sprite rendering with camera transforms. Vue excels at reactive UI. Mixing them (canvas below, DOM above) is a proven pattern for 2D browser games.

**Alternatives considered**:
- *Phaser*: Full game framework — overkill when we only need rendering (simulation is in Rust).
- *Pure Canvas 2D API*: No WebGL acceleration, manual sprite batching needed.
- *Vue + CSS grid for tiles*: DOM rendering at 50×50 tiles is too slow for smooth pan/zoom.

### D5: Vite build pipeline

**Choice**: Vite with `vite-plugin-wasm` for WASM imports in the worker, standard Vue SFC support.

**Rationale**: Vite has first-class support for workers (`?worker`), Vue 3, and TypeScript. The WASM plugin handles `.wasm` content-type and instantiation.

### D6: Tile world representation

**Choice**: Flat `Vec<TileType>` of size `WIDTH * HEIGHT` in Rust, indexed by `y * WIDTH + x`. Sent to JS as a serialized array on initialization and only when tiles change.

**Rationale**: Simple, cache-friendly, and easy to serialize. 2500 tiles is tiny — no need for chunks or spatial indexing.

### D7: A* pathfinding

**Choice**: Standard A* with a binary heap on the flat tile grid. Tiles have a `walkable` flag (water = not walkable). Path is computed per-colonist when a new task is assigned.

**Rationale**: 50×50 grid means worst-case ~2500 nodes — trivial for A*. No need for JPS, flow fields, or nav meshes at this scale.

## Risks / Trade-offs

- **[WASM build complexity]** → Developers need `wasm-pack` and Rust toolchain installed. Mitigated by documenting setup and providing a build script.
- **[Serialization overhead on every tick]** → Sending full world state as JSON at 20 ticks/sec for a 50×50 world. Mitigated by the small world size (≈50KB serialized); can switch to delta updates later.
- **[Worker message latency]** → `postMessage` has ~1ms overhead. At 50ms tick intervals this is negligible (2% overhead). Would matter more at higher tick rates.
- **[PixiJS + Vue coordination]** → PixiJS canvas captures mouse events that Vue also wants (clicks on colonists vs. clicks on HUD). Mitigated by using DOM overlay for UI and passing canvas click coordinates through PixiJS's event system.
- **[No save/load in v1]** → If the page refreshes, game state is lost. Acceptable for v1; serialization via `serde` makes future save/load straightforward.
