# Capability: Game Core (Rust/WASM)

## Purpose

Rust-based game simulation engine compiled to WebAssembly. Manages the 50×50 tile world, colonist AI (needs-driven behavior), building placement, A* pathfinding, and deterministic tick-based updates. Exposes a minimal WASM API (`init`, `tick`, `push_event`, `drain_events`).

## Requirements

### Requirement: World Grid

The system SHALL maintain a 50×50 flat tile grid where each tile is one of Water, Sand, or Grass.

#### Scenario: World generation produces connected walkable area
- **GIVEN** a new game is initialized
- **WHEN** the world is generated
- **THEN** tiles are randomized with weighted distribution (~15% Water, ~20% Sand, ~65% Grass) and flood-fill validation ensures ≥90% of walkable tiles form a single connected component

#### Scenario: Tile queries respect bounds
- **GIVEN** a generated world
- **WHEN** `get_tile(x, y)` is called with out-of-bounds coordinates
- **THEN** it returns Water (impassable)

### Requirement: Colonist Needs & AI

The system SHALL spawn 3 colonists at random walkable positions with Food and Sleep needs initialized to 1.0, decaying over time.

#### Scenario: Needs decay each tick
- **GIVEN** a colonist with Food=1.0 and Sleep=1.0
- **WHEN** a tick occurs with dt=1.0
- **THEN** Food decreases by `FOOD_DECAY_RATE * dt` and Sleep decreases by `SLEEP_DECAY_RATE * dt`

#### Scenario: Colonist seeks food when hungry
- **GIVEN** an idle colonist with Food below the threshold (0.5)
- **AND** a BerryBush building exists on the map
- **WHEN** the task assignment system runs
- **THEN** the colonist is assigned MovingToFood task with a path to the nearest BerryBush

#### Scenario: Colonist seeks bed when tired
- **GIVEN** an idle colonist with Sleep below the threshold (0.5)
- **AND** a Bed building exists on the map
- **WHEN** the task assignment system runs
- **THEN** the colonist is assigned MovingToBed task with a path to the nearest Bed

#### Scenario: Colonist eats at berry bush
- **GIVEN** a colonist with task=Eating adjacent to a BerryBush
- **WHEN** ticks occur
- **THEN** Food increases by `RESTORE_RATE * dt` until reaching 0.95, then task resets to Idle

#### Scenario: Colonist sleeps at bed
- **GIVEN** a colonist with task=Sleeping adjacent to a Bed
- **WHEN** ticks occur
- **THEN** Sleep increases by `RESTORE_RATE * dt` until reaching 0.95, then task resets to Idle

#### Scenario: Idle wandering
- **GIVEN** an idle colonist with no urgent needs
- **WHEN** the task assignment system runs
- **THEN** the colonist walks to a random nearby walkable tile

### Requirement: Building Placement

The system SHALL allow placement of Wall, Bed, and BerryBush buildings via PlaceBuilding events.

#### Scenario: Valid placement
- **GIVEN** a PlaceBuilding event for a walkable, unoccupied tile
- **WHEN** the event is processed
- **THEN** the building entity is created and a BuildingPlaced outgoing event is emitted

#### Scenario: Invalid placement on water
- **GIVEN** a PlaceBuilding event targeting a Water tile
- **WHEN** the event is processed
- **THEN** an Error outgoing event is emitted with an appropriate message

#### Scenario: Invalid placement on occupied tile
- **GIVEN** a PlaceBuilding event targeting a tile already occupied by another building
- **WHEN** the event is processed
- **THEN** an Error outgoing event is emitted

### Requirement: A* Pathfinding

The system SHALL provide A* pathfinding on the tile grid using Manhattan distance heuristic.

#### Scenario: Path found to walkable target
- **GIVEN** a start position and a walkable target position
- **WHEN** `find_path` is called
- **THEN** it returns the shortest path avoiding Water tiles and Wall buildings

#### Scenario: Adjacent targeting for buildings
- **GIVEN** a colonist needing to interact with a building
- **WHEN** `find_path_adjacent` is called
- **THEN** it returns a path to the closest walkable tile adjacent to the building

#### Scenario: Unreachable target
- **GIVEN** a target completely surrounded by water or walls
- **WHEN** pathfinding is attempted
- **THEN** it returns None and the colonist remains Idle

### Requirement: Tick-Based Simulation

The system SHALL process game logic in discrete ticks via a `tick(dt)` function.

#### Scenario: Tick processing order
- **GIVEN** the game engine is running
- **WHEN** `tick(dt)` is called
- **THEN** it processes in order: incoming events → needs decay → task assignment → movement → task actions → emit WorldSnapshot

#### Scenario: WorldSnapshot emission
- **GIVEN** a tick has completed
- **WHEN** outgoing events are drained
- **THEN** a WorldSnapshot containing all tile data, colonist states (id, name, position, needs, task), and building states is included
