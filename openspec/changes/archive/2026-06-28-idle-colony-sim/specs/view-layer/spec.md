## ADDED Requirements

### Requirement: Three-layer tile rendering
The system SHALL render the game world using PixiJS 8 with three distinct layers: terrain (bottom), blueprints (middle), and entities (top). The terrain layer SHALL render each tile as a colored rectangle (Water=blue, Sand=yellow, Grass=green). The entity layer SHALL render colonists and buildings as distinct sprites or shapes.

#### Scenario: Terrain renders on load
- **WHEN** the first WorldSnapshot is received from the worker
- **THEN** a 50×50 grid of colored tiles SHALL be rendered on the terrain layer corresponding to tile types

#### Scenario: Entities render above terrain
- **WHEN** colonists and buildings are present in the WorldSnapshot
- **THEN** colonist sprites and building sprites SHALL be rendered on the entity layer, visually above the terrain

### Requirement: Camera pan and zoom
The system SHALL support camera controls: panning by clicking and dragging on the canvas, and zooming via mouse scroll wheel. The camera SHALL clamp to the world bounds (cannot scroll past the 50×50 grid). Zoom SHALL have minimum and maximum limits.

#### Scenario: Pan the camera
- **WHEN** the user clicks and drags on the canvas
- **THEN** the visible world area SHALL move in the opposite direction of the drag

#### Scenario: Zoom in/out
- **WHEN** the user scrolls the mouse wheel
- **THEN** the camera zoom level SHALL increase or decrease, clamped to min/max bounds

#### Scenario: Camera clamps to world bounds
- **WHEN** the user pans or zooms beyond the world edge
- **THEN** the camera SHALL stop at the world boundary and not show empty space

### Requirement: HUD with pause and speed controls
The system SHALL display a HUD overlay (DOM-based, rendered by Vue) with: a Pause/Resume toggle button and speed control buttons (1x, 2x, 3x). Clicking Pause SHALL send a Pause event to the worker. Speed buttons SHALL send a SetSpeed event with the corresponding multiplier.

#### Scenario: Pause the game
- **WHEN** the user clicks the Pause button
- **THEN** a Pause IncomingEvent SHALL be sent to the worker and the button SHALL display "Resume"

#### Scenario: Resume the game
- **WHEN** the user clicks the Resume button while paused
- **THEN** a Resume IncomingEvent SHALL be sent to the worker and the button SHALL display "Pause"

#### Scenario: Change game speed
- **WHEN** the user clicks a speed button (1x, 2x, or 3x)
- **THEN** a SetSpeed IncomingEvent with the selected multiplier SHALL be sent to the worker, and the active speed button SHALL be visually highlighted

### Requirement: Building toolbar
The system SHALL display a toolbar (DOM-based, rendered by Vue) with buttons for each buildable type: Wall, Bed, BerryBush. Selecting a tool enters "placement mode." In placement mode, hovering over the canvas SHALL show a semi-transparent preview (blueprint) on the blueprint layer. Clicking a valid tile SHALL send a PlaceBuilding event to the worker.

#### Scenario: Enter placement mode
- **WHEN** the user clicks the "Bed" button in the toolbar
- **THEN** the cursor enters placement mode for Bed, and hovering over tiles shows a blueprint preview on the blueprint layer

#### Scenario: Place a building
- **WHEN** the user clicks a valid tile while in placement mode
- **THEN** a PlaceBuilding IncomingEvent SHALL be sent to the worker with the selected building type and tile coordinates

#### Scenario: Cancel placement mode
- **WHEN** the user presses Escape or clicks the active tool button again
- **THEN** placement mode SHALL be deactivated and the blueprint preview removed

### Requirement: Colonist info panel
The system SHALL display an information panel (DOM-based, rendered by Vue) when a colonist is clicked. The panel SHALL show: colonist name, current Food and Sleep values (as progress bars), current task description, and grid position (x, y). Clicking another colonist SHALL switch the panel. Clicking empty space SHALL close the panel.

#### Scenario: Select a colonist
- **WHEN** the user clicks on a colonist sprite on the canvas
- **THEN** an info panel appears showing the colonist's name, Food bar, Sleep bar, current task, and position

#### Scenario: Deselect colonist
- **WHEN** the user clicks on empty terrain while a colonist is selected
- **THEN** the info panel SHALL close

#### Scenario: Panel updates in real-time
- **WHEN** a colonist is selected and new WorldSnapshot data arrives
- **THEN** the info panel SHALL update to reflect current need values, task, and position
