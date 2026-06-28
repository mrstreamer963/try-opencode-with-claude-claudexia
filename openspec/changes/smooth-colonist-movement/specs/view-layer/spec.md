## MODIFIED Requirements

### Requirement: PixiJS Canvas Rendering

The system SHALL render the game world using PixiJS with layered containers for terrain, buildings, blueprints, and entities. Terrain and buildings SHALL be redrawn only when a new `WorldSnapshot` arrives; entities and the placement blueprint SHALL be redrawn every Pixi ticker frame to support smooth interpolation.

#### Scenario: Terrain rendering
- **GIVEN** a WorldSnapshot with 50×50 tile data
- **WHEN** the renderer processes it
- **THEN** each tile is drawn as a colored rectangle (Water=blue #2196F3, Sand=yellow #F5D68D, Grass=green #66BB6A) with subtle grid lines

#### Scenario: Entity rendering
- **GIVEN** a WorldSnapshot with colonist states
- **WHEN** the renderer draws entities
- **THEN** each colonist appears as a colored circle with a task indicator (pink dot for Eating, purple dot for Sleeping) drawn at an interpolated float position between `(x, y)` and `(next_x, next_y)` using `move_progress`; when `next_x`/`next_y` are null the colonist is drawn exactly at `(x, y)`

#### Scenario: Building rendering
- **GIVEN** buildings exist in the snapshot
- **WHEN** the renderer draws the building layer
- **THEN** each building is drawn as a colored square (Wall=gray, Bed=brown, BerryBush=pink) inset by 1px with a border

#### Scenario: Blueprint preview
- **GIVEN** placement mode is active with a building type selected
- **WHEN** the user hovers over a tile
- **THEN** a semi-transparent preview of the building appears at the hovered tile position

#### Scenario: Render optimization
- **GIVEN** no new WorldSnapshot has arrived since the last render
- **WHEN** the render loop ticks
- **THEN** the terrain and building layers are NOT re-drawn, but the entity and blueprint layers ARE re-drawn so that interpolated colonist motion remains smooth between snapshots

#### Scenario: Smooth colonist motion between snapshots
- **GIVEN** a colonist whose latest snapshot reports `move_progress = 0.4` toward `(next_x, next_y)`
- **WHEN** the Pixi ticker fires multiple frames before the next snapshot arrives
- **THEN** the colonist is drawn at the same interpolated position each frame (no visible jitter), and updates only when the next snapshot changes `move_progress`
