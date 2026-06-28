## ADDED Requirements

### Requirement: Tile world generation
The system SHALL generate a 50×50 tile grid on initialization. Each tile SHALL be one of: Water, Sand, or Grass. The tile distribution SHALL be randomized but produce a playable map (connected walkable area). Water tiles SHALL NOT be walkable.

#### Scenario: World initializes with correct dimensions
- **WHEN** the game core initializes
- **THEN** a 50×50 tile grid is created with each tile assigned a type (Water, Sand, or Grass)

#### Scenario: Water tiles block movement
- **WHEN** a colonist attempts to path through a Water tile
- **THEN** the A* pathfinder SHALL treat that tile as impassable and route around it

### Requirement: Colonist entities with needs
The system SHALL spawn 3 colonist entities at game start. Each colonist SHALL have two needs: Food (0.0–1.0) and Sleep (0.0–1.0). Both needs SHALL initialize at 1.0 and decay over time at a configurable rate per tick.

#### Scenario: Colonists spawn at game start
- **WHEN** the game core initializes
- **THEN** 3 colonist entities are created at random walkable positions with Food=1.0 and Sleep=1.0

#### Scenario: Needs decay over time
- **WHEN** a game tick executes
- **THEN** each colonist's Food and Sleep values SHALL decrease by the configured decay rate multiplied by delta time

### Requirement: Automatic task assignment
The system SHALL automatically assign tasks to idle colonists based on their lowest need. When Food drops below a threshold, the colonist SHALL seek a BerryBush and eat. When Sleep drops below a threshold, the colonist SHALL seek a Bed and sleep. Eating SHALL restore Food; sleeping SHALL restore Sleep.

#### Scenario: Hungry colonist seeks food
- **WHEN** a colonist's Food need drops below the hunger threshold AND a BerryBush exists
- **THEN** the colonist SHALL be assigned a task to pathfind to the nearest BerryBush and eat, restoring Food

#### Scenario: Tired colonist seeks bed
- **WHEN** a colonist's Sleep need drops below the sleep threshold AND a Bed exists
- **THEN** the colonist SHALL be assigned a task to pathfind to the nearest Bed and sleep, restoring Sleep

#### Scenario: Idle colonist with no urgent needs
- **WHEN** a colonist has no needs below threshold and no active task
- **THEN** the colonist SHALL wander randomly to a nearby walkable tile

### Requirement: A* pathfinding
The system SHALL compute paths using the A* algorithm on the tile grid. Paths SHALL avoid Water tiles and reach the target tile adjacent to the destination entity. If no valid path exists, the task SHALL be cancelled.

#### Scenario: Path found to target
- **WHEN** a colonist is assigned a task with a reachable target
- **THEN** the system computes an A* path from the colonist's current tile to a tile adjacent to the target

#### Scenario: No valid path
- **WHEN** a colonist is assigned a task but no walkable path exists to the target
- **THEN** the task SHALL be cancelled and the colonist returns to idle state

### Requirement: Building placement
The system SHALL support placing buildings on the tile grid via an IncomingEvent. Supported building types: Wall (blocks movement), Bed (sleep station), BerryBush (food source). Buildings SHALL be placed instantly (no resource cost in v1). A building SHALL NOT be placed on a Water tile or on a tile already occupied by another building.

#### Scenario: Place a Bed on a valid tile
- **WHEN** a PlaceBuilding event is received for a Bed at a walkable, unoccupied tile
- **THEN** a Bed entity is created at that tile position

#### Scenario: Reject placement on Water
- **WHEN** a PlaceBuilding event is received for a tile that is Water
- **THEN** the building SHALL NOT be placed and an error event SHALL be emitted

#### Scenario: Reject placement on occupied tile
- **WHEN** a PlaceBuilding event is received for a tile already occupied by a building
- **THEN** the building SHALL NOT be placed and an error event SHALL be emitted

### Requirement: Tick-based simulation
The system SHALL expose a `tick(dt: f32)` function callable from the WebWorker. Each tick SHALL: decay colonist needs, run task assignment, execute pathfinding movement, and update all entity states. The tick function SHALL process all pending `IncomingEvent` messages before advancing simulation.

#### Scenario: Tick advances simulation
- **WHEN** `tick(dt)` is called with a delta time
- **THEN** all systems execute in order: process incoming events → decay needs → assign tasks → move colonists along paths → emit outgoing events

#### Scenario: Multiple incoming events in one tick
- **WHEN** multiple IncomingEvent messages have been queued before a tick
- **THEN** all queued events SHALL be processed before the simulation step runs

### Requirement: Event-based communication
The system SHALL accept `IncomingEvent` variants (PlaceBuilding, SetSpeed, Pause, Resume) and emit `OutgoingEvent` variants (WorldSnapshot, ColonistUpdated, BuildingPlaced, Error). Events SHALL be serialized as JSON via `serde_json` and exchanged through `wasm-bindgen` functions.

#### Scenario: Incoming PlaceBuilding event
- **WHEN** a PlaceBuilding IncomingEvent is received with building type and coordinates
- **THEN** the system attempts placement and emits either BuildingPlaced or Error

#### Scenario: WorldSnapshot emitted after tick
- **WHEN** a tick completes
- **THEN** the system SHALL emit a WorldSnapshot OutgoingEvent containing tile data, colonist states (position, needs, current task), and building positions
