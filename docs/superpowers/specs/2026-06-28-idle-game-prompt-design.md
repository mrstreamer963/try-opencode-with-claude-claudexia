# Idle Game Prompt Design (RimWorld-like)

## Tech Stack
- **Core:** Rust + `bevy_ecs` + `wasm-bindgen` + `serde`/`bincode` → WASM
- **View:** Vite 6 + Vue 3 + TypeScript + Pinia + PixiJS 8
- **Integration:** WebWorker bridge, game loop in Worker, render in Main thread

## Architecture: 3 modular prompts

The full prompt is split into 3 standalone prompts, each targeting one layer:

1. **Game Core** (Rust/WASM) — ECS logic, tile map, colonists, needs, building
2. **View Layer** (Vue/PixiJS) — rendering, camera, HUD, blueprint placement
3. **Integration** (Worker Bridge) — connects core ↔ view via typed messaging

---

## Prompt #1: Game Core (Rust + bevy_ecs → WASM)

**Objective:** Generate the core engine of an idle colony sim.
Runs inside a WebWorker, compiled to WASM. No direct rendering — only logic + state.

### Dependencies

```toml
[dependencies]
bevy_ecs = "0.15"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
wasm-bindgen = "0.2"
```

### Architecture

- `bevy_ecs::World` as data store. No Schedule — manual `tick()` calling systems in order.
- `Events<T>` as event bus between JS and Rust: `IncomingEvent` / `OutgoingEvent`.
- All data crossing WASM boundary: `#[derive(Serialize, Deserialize)]`.

### Data Types

```rust
#[derive(Serialize, Deserialize)]
enum TileType { Grass, Wall }

#[derive(Component, Serialize, Deserialize)]
enum NeedKind { Food, Sleep }

#[derive(Component, Serialize, Deserialize)]
struct Need { kind: NeedKind, value: f32 }

#[derive(Serialize, Deserialize)]
enum TaskKind { Build(IVec2, BlueprintKind), Eat, Sleep }

#[derive(Component, Serialize, Deserialize)]
struct Task { id: u64, kind: TaskKind, assigned_to: Option<Entity>, completed: bool }

#[derive(Serialize, Deserialize)]
enum BlueprintKind { Wall, Bed, BerryBush }

#[derive(Component, Serialize, Deserialize)]
struct Blueprint { pos: IVec2, kind: BlueprintKind, progress: f32 }

#[derive(Component, Serialize, Deserialize)]
struct Position { x: f32, y: f32 }

#[derive(Component, Serialize, Deserialize)]
struct Colonist { speed: f32, task_queue: Vec<u64>, color: String }
```

### Events

```rust
#[derive(Serialize, Deserialize)]
enum IncomingEvent {
    SetBlueprint { x: i32, y: i32, kind: BlueprintKind },
    RemoveBlueprint { x: i32, y: i32 },
    SetSpeed { multiplier: f32 },
    TogglePause,
}

#[derive(Serialize, Deserialize)]
enum OutgoingEvent {
    StateSnapshot { json: String },
    BuildingComplete { x: i32, y: i32, kind: BlueprintKind },
}
```

### Resources

```rust
struct GameTime {
    speed_multiplier: f32,  // 1.0, 5.0, 10.0
    paused: bool,
}

struct Map {
    grid: [[Tile; 50]; 50],
}
```

### Systems (called sequentially from `tick(dt: f64)`)

1. **needs_system** — decrement `Need.value` per dt. Food drops faster. If food < 30 → spawn `Task::Eat` targeting nearest BerryBush. If sleep < 20 → spawn `Task::Sleep` targeting nearest unoccupied Bed.
2. **task_assignment_system** — each idle colonist (no task_queue) finds nearest unclaimed `Task`. Set `assigned_to`. Tie-break: closest first.
3. **movement_system** — colonists with assigned tasks move along A* path. Interpolate position between tiles at `colonist.speed * dt * speed_mult`.
4. **construction_system** — when colonist reaches Blueprint position, increment `progress` per dt. At `progress >= 1.0`: despawn Blueprint, spawn Building, emit `BuildingComplete`.
5. **cleanup_system** — remove completed tasks. On BuildingComplete: update TileType (Wall → !walkable).

### Pathfinding (A*)

- Grid 50×50
- Obstacles: Wall tiles, other colonists (simplified: passable with distance penalty)
- Result: `Vec<IVec2>` — path from start to end
- Colonist stores `path: Vec<IVec2>` and `path_index: usize`, lerps between tile centers

### WASM API

```rust
#[wasm_bindgen]
pub fn init() -> JsValue;  // returns initial StateSnapshot

#[wasm_bindgen]
pub fn tick(dt: f64, speed_mult: f32, paused: bool) -> JsValue;  // returns StateSnapshot

#[wasm_bindgen]
pub fn handle_event(json: &str) -> JsValue;  // processes IncomingEvent, returns StateSnapshot
```

### File Structure

```
game-core/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── components.rs
│   ├── resources.rs
│   ├── events.rs
│   ├── systems/
│   │   ├── needs.rs
│   │   ├── tasks.rs
│   │   ├── movement.rs
│   │   ├── construction.rs
│   │   └── cleanup.rs
│   ├── pathfinding.rs
│   └── map.rs
```

---

## Prompt #2: View Layer (Vite + Vue 3 + PixiJS 8)

**Objective:** Generate the client-side rendering and UI.

### Tech Stack
- Vite 6 + Vue 3 + TypeScript + Pinia
- PixiJS 8 (Canvas2D/WebGL renderer)
- WebWorker bridge to WASM core

### Vue Components

**App.vue** — layout: GameCanvas + HUD + BlueprintToolbar

**GameCanvas.vue** — mounts PIXI Application into a div ref. Lifecycle: create on mount, destroy on unmount.

**GameHud.vue**
- Displays: speed, pause state, colonist count, total blueprints
- Buttons: Pause/Resume, Speed (x1/x5/x10)
- All values from Pinia store (updated on each snapshot)

**BlueprintToolbar.vue**
- 3 buttons: Wall, Bed, BerryBush
- Active tool stored in Pinia
- Click on tile in game → sends `IncomingEvent::SetBlueprint`

### PixiJS Layers (draw order)

1. **TileLayer** — 50×50 grid. Each tile = `PIXI.Graphics` (green Grass, gray Wall). On tile change: redraw only that tile.
2. **BlueprintLayer** — semi-transparent overlays at blueprint positions. Wall: gray semi square. Bed: brown rectangle. BerryBush: green circle.
3. **EntityLayer** — colonists as circles (diameter 0.6 tile). Different colors per colonist. Lerp position each frame for smooth movement.
4. **GridOverlay (optional)** — thin grid lines

### Camera

- Pan: mouse drag (PIXI `eventMode`)
- Zoom: scroll wheel, clamp 0.5×…3×
- Bounds: camera cannot pan outside map edges

### Blueprint Placement Flow

1. In blueprint mode: hover highlights tile (semi-transparent square)
2. Click → `bridge.placeBlueprint(x, y, kind)` → `worker.postMessage(event)`
3. Worker processes → snapshot returned → PIXI updates

### State Update Flow

```
Worker message → bridge.onSnapshot(snapshot)
  → tileLayer.update(snapshot.tiles)
  → entityLayer.update(snapshot.colonists)
  → blueprintLayer.update(snapshot.blueprints)
  → gameStore.updateStats(snapshot.stats)
```

### File Structure

```
view/
├── index.html
├── vite.config.ts
├── tsconfig.json
├── package.json
├── src/
│   ├── main.ts
│   ├── App.vue
│   ├── components/
│   │   ├── GameCanvas.vue
│   │   ├── GameHud.vue
│   │   └── BlueprintToolbar.vue
│   ├── pixi/
│   │   ├── GameRenderer.ts
│   │   ├── TileLayer.ts
│   │   ├── EntityLayer.ts
│   │   ├── BlueprintLayer.ts
│   │   └── camera.ts
│   ├── worker/
│   │   └── bridge.ts
│   ├── stores/
│   │   └── game-store.ts
│   └── types.ts
```

---

## Prompt #3: Integration & Worker Bridge

**Objective:** Connect WASM core with View layer via WebWorker.

### WebWorker (game.worker.ts)

```
On init:
  1. Import WASM from game-core/pkg
  2. Call wasm.init()
  3. Set up onmessage handler

On message (IncomingEvent):
  → wasm.handle_event(json)

Game loop:
  let last = performance.now()
  On each tick (setInterval 50ms = 20 t/s):
    now = performance.now()
    dt = (now - last) / 1000
    snapshot = wasm.tick(dt, speed, paused)
    postMessage({ type: 'snapshot', data: snapshot })
    last = now
```

### Frame rate independence

- Game loop: `setInterval(50ms)` in Worker — 20 logic ticks/sec
- Render loop: `requestAnimationFrame` on main thread — takes latest snapshot, interpolates entity positions for smooth visuals
- Worker ticks independently; if tab is backgrounded, main thread just picks up latest snapshot when rAF resumes

### Bridge (bridge.ts)

```typescript
export class GameBridge {
  private worker: Worker
  private onSnapshot: (snapshot: Snapshot) => void

  postCommand(event: IncomingEvent): void
  setCallback(fn: (snapshot: Snapshot) => void): void
  destroy(): void
}
```

### Snapshot Type

```typescript
export interface Snapshot {
  tiles: { x: number; y: number; kind: 'Grass' | 'Wall' }[]
  colonists: {
    id: number
    x: number
    y: number
    color: string
    task: string | null
  }[]
  blueprints: {
    x: number
    y: number
    kind: 'Wall' | 'Bed' | 'BerryBush'
    progress: number
  }[]
  stats: {
    colonistsCount: number
    blueprintsCount: number
  }
}
```

### Bundle Config

Vite config requires:
- `vite-plugin-wasm` — for WASM module support
- `vite-plugin-top-level-await` — for top-level await in WASM init
- Worker: `new Worker(new URL('./game.worker.ts', import.meta.url), { type: 'module' })`

### File Structure

```
worker/
├── game.worker.ts
├── wasm-bridge.ts
└── snapshot.ts
```

---

## Gameplay Summary (v1 scope)

- **3 colonists** on a 50×50 tile map
- **Needs:** Food (drains faster), Sleep (drains slower)
  - Food < 30 → auto-task: eat at nearest BerryBush
  - Sleep < 20 → auto-task: sleep at nearest Bed
  - **Colonists do not die** at 0 need (idle-safe)
- **Building:** Player places blueprints (Wall, Bed, BerryBush). Colonists auto-build.
- **No resource costs** for building (simplified for v1)
- **Time:** Realtime with pause, x1, x5, x10 speed
- **Movement:** Smooth (not tile-snapping), A* pathfinding
