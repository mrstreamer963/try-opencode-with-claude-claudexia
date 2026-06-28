# Capability: View Layer

## Purpose

Browser-based rendering and UI layer using PixiJS 8 for the game canvas and Vue 3 components for HUD overlays. Renders the tile world, colonists, and buildings from WorldSnapshot data, and provides controls for game speed, building placement, and colonist inspection.

## Requirements

### Requirement: PixiJS Canvas Rendering

The system SHALL render the game world using PixiJS with layered containers for terrain, buildings, blueprints, and entities.

#### Scenario: Terrain rendering
- **GIVEN** a WorldSnapshot with 50×50 tile data
- **WHEN** the renderer processes it
- **THEN** each tile is drawn as a colored rectangle (Water=blue #2196F3, Sand=yellow #F5D68D, Grass=green #66BB6A) with subtle grid lines

#### Scenario: Entity rendering
- **GIVEN** a WorldSnapshot with colonist states
- **WHEN** the renderer draws entities
- **THEN** each colonist appears as a colored circle at their grid position with a task indicator (pink dot for Eating, purple dot for Sleeping)

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
- **THEN** the renderer skips re-drawing

### Requirement: Camera Controls

The system SHALL support pan and zoom controls on the game canvas.

#### Scenario: Camera pan
- **GIVEN** the user clicks and drags on the canvas
- **WHEN** the pointer moves
- **THEN** the PixiJS stage position translates by the drag delta

#### Scenario: Camera zoom
- **GIVEN** the user scrolls the mouse wheel
- **WHEN** the wheel event fires
- **THEN** the stage scale adjusts (0.5× to 5× range) zooming toward the mouse cursor position

### Requirement: HUD Controls

The system SHALL provide Pause/Resume and Speed control buttons.

#### Scenario: Pause toggle
- **GIVEN** the game is running
- **WHEN** the user clicks the pause button
- **THEN** a Pause event is sent and the button shows a play icon; clicking again sends Resume

#### Scenario: Speed selection
- **GIVEN** the HUD shows speed buttons 1×, 2×, 3×
- **WHEN** the user clicks 2×
- **THEN** a SetSpeed event with speed=2 is sent and the 2× button shows as active

### Requirement: Building Toolbar

The system SHALL provide a toolbar for selecting building types to place.

#### Scenario: Toggle placement mode
- **GIVEN** no building is selected
- **WHEN** the user clicks the Wall button
- **THEN** placement mode activates for Wall; clicking again deactivates it

#### Scenario: Escape cancels placement
- **GIVEN** placement mode is active
- **WHEN** the user presses Escape
- **THEN** placement mode is cancelled

#### Scenario: Tile click places building
- **GIVEN** placement mode is active with BerryBush selected
- **WHEN** the user clicks on a tile at (x, y)
- **THEN** a PlaceBuilding event is sent with buildingType=BerryBush and the clicked coordinates

### Requirement: Colonist Info Panel

The system SHALL display a detail panel when a colonist is clicked.

#### Scenario: Open panel on click
- **GIVEN** the user clicks on a tile containing a colonist
- **WHEN** the click is processed
- **THEN** the ColonistInfoPanel opens showing the colonist's name, position, current task badge, and Food/Sleep progress bars

#### Scenario: Real-time updates
- **GIVEN** the panel is open for a colonist
- **WHEN** new WorldSnapshots arrive
- **THEN** the panel updates with the colonist's latest needs and task values

#### Scenario: Close panel
- **GIVEN** the panel is open
- **WHEN** the user clicks on empty space or presses Escape
- **THEN** the panel closes
