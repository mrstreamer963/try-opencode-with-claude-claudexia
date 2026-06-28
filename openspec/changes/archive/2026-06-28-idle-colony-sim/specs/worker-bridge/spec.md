## ADDED Requirements

### Requirement: WebWorker WASM initialization
The system SHALL create a dedicated WebWorker that loads and initializes the game-core WASM module. The worker SHALL use `vite-plugin-wasm` for WASM instantiation. Upon successful initialization, the worker SHALL send a "Ready" event to the main thread.

#### Scenario: Worker initializes successfully
- **WHEN** the application starts and the worker script loads
- **THEN** the WASM module SHALL be instantiated inside the worker and a Ready event SHALL be posted to the main thread

#### Scenario: WASM initialization failure
- **WHEN** the WASM module fails to load (e.g., network error)
- **THEN** the worker SHALL post an Error event with a descriptive message to the main thread

### Requirement: Game loop in worker
The system SHALL run a game loop inside the WebWorker using `setInterval` at 50ms intervals (20 ticks/sec base rate). Each interval SHALL call the WASM `tick(dt)` function with the elapsed delta time multiplied by the current speed multiplier. The loop SHALL be pausable via Pause/Resume events.

#### Scenario: Game loop ticks at expected rate
- **WHEN** the game is running at 1x speed
- **THEN** `tick()` SHALL be called approximately every 50ms with dt ≈ 0.05

#### Scenario: Speed multiplier affects tick
- **WHEN** the speed is set to 2x
- **THEN** `tick()` SHALL be called with dt multiplied by 2 (simulation advances faster)

#### Scenario: Pause stops ticking
- **WHEN** a Pause event is received
- **THEN** the game loop SHALL stop calling `tick()` until a Resume event is received

### Requirement: Main-thread event dispatch
The system SHALL provide a TypeScript bridge module on the main thread that: (1) sends IncomingEvent messages to the worker via `postMessage`, and (2) receives OutgoingEvent messages from the worker and dispatches them to registered listeners (Vue composable or event bus).

#### Scenario: Send PlaceBuilding to worker
- **WHEN** the view layer calls `bridge.send({ type: 'PlaceBuilding', buildingType: 'Bed', x: 10, y: 15 })`
- **THEN** the message SHALL be posted to the worker via `postMessage`

#### Scenario: Receive WorldSnapshot from worker
- **WHEN** the worker posts an OutgoingEvent of type WorldSnapshot
- **THEN** the bridge SHALL invoke all registered listeners with the parsed WorldSnapshot data

### Requirement: Render synchronization
The main thread SHALL use `requestAnimationFrame` to drive PixiJS rendering. On each animation frame, the renderer SHALL use the latest WorldSnapshot received from the worker. If no new snapshot has arrived since the last frame, the previous snapshot SHALL be reused (no re-render of unchanged state).

#### Scenario: Render uses latest snapshot
- **WHEN** a new WorldSnapshot arrives between animation frames
- **THEN** the next `requestAnimationFrame` callback SHALL render using the new snapshot data

#### Scenario: No new data between frames
- **WHEN** no new WorldSnapshot arrives between two consecutive animation frames
- **THEN** the renderer SHALL skip updating game objects (reuse previous frame)

### Requirement: Typed message protocol
The system SHALL define a shared TypeScript type definition for all message types exchanged between main thread and worker. IncomingEvent types SHALL include: PlaceBuilding, SetSpeed, Pause, Resume. OutgoingEvent types SHALL include: Ready, WorldSnapshot, BuildingPlaced, Error. Both sides SHALL use these types for type-safe message handling.

#### Scenario: Type-safe message construction
- **WHEN** the main thread constructs an IncomingEvent
- **THEN** TypeScript SHALL enforce that the message conforms to one of the defined IncomingEvent variants with all required fields

#### Scenario: Type-safe message reception
- **WHEN** the worker receives a postMessage
- **THEN** the message SHALL be parsed and matched against the IncomingEvent type union for dispatch
