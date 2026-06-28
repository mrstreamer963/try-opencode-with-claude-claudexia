## MODIFIED Requirements

### Requirement: Colonist Needs & AI

The system SHALL spawn 3 colonists at random walkable positions with Food and Sleep needs initialized to 1.0, decaying over time. Movement along assigned paths SHALL accumulate float `move_progress` over time at a base rate of 3 cells/sec on Grass, scaled by per-tile `move_cost` (Grass = 1.0, Sand = 1.8, Water = ∞ / impassable).

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

#### Scenario: Smooth cell-to-cell movement
- **GIVEN** a colonist with a `Path` whose `current_step` targets cell `(nx, ny)`
- **WHEN** `tick(dt)` runs
- **THEN** `move_progress += dt * BASE_MOVE_SPEED / move_cost(tile_at(nx, ny))`, and the colonist's logical `Position` stays at the "from" cell until `move_progress >= 1.0`

#### Scenario: Crossing a cell boundary
- **GIVEN** a colonist whose `move_progress` reaches or exceeds 1.0 during a tick
- **WHEN** the movement step completes
- **THEN** `Position` advances to the next step, `current_step` increments, and any excess progress (`move_progress - 1.0`) is carried into the following cell so motion does not stall at boundaries

#### Scenario: Per-terrain movement speed
- **GIVEN** the same colonist moving on Grass vs. Sand
- **WHEN** equal `dt` elapses
- **THEN** progress on Sand accrues at ~1/1.8 the rate of Grass, making Sand crossings visibly slower

#### Scenario: Path completion
- **GIVEN** a colonist whose `current_step` reaches `steps.len()`
- **WHEN** the movement system runs
- **THEN** the `Path` component is removed and the colonist's task transitions (MovingToFood → Eating, MovingToBed → Sleeping, Wandering → Idle)

### Requirement: Tick-Based Simulation

The system SHALL process game logic in discrete ticks via a `tick(dt)` function.

#### Scenario: Tick processing order
- **GIVEN** the game engine is running
- **WHEN** `tick(dt)` is called
- **THEN** it processes in order: incoming events → needs decay → task assignment → movement → task actions → emit WorldSnapshot

#### Scenario: WorldSnapshot emission
- **GIVEN** a tick has completed
- **WHEN** outgoing events are drained
- **THEN** a WorldSnapshot containing all tile data, colonist states (id, name, position, `next_x`, `next_y`, `move_progress`, needs, task), and building states is included

#### Scenario: Snapshot fields for stationary colonist
- **GIVEN** a colonist with no `Path` component (idle, eating, sleeping, or just finished moving)
- **WHEN** a snapshot is emitted
- **THEN** `next_x = null`, `next_y = null`, and `move_progress = 0.0` for that colonist

#### Scenario: Snapshot fields for moving colonist
- **GIVEN** a colonist with a `Path` whose `current_step` targets `(nx, ny)` with progress `p`
- **WHEN** a snapshot is emitted
- **THEN** `next_x = nx`, `next_y = ny`, and `move_progress = p` for that colonist; `x`/`y` reflect the integer "from" cell

#### Scenario: Pause freezes movement progress
- **GIVEN** a colonist mid-transition with `move_progress = 0.4`
- **WHEN** the game is paused (worker stops calling `tick`)
- **THEN** subsequent snapshots (if any) report the same `move_progress = 0.4` until the game resumes
