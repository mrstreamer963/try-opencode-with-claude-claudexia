# Capability: Worker Bridge

## Purpose

Web Worker layer that loads the Rust/WASM game core off the main thread, runs the simulation tick loop, and relays JSON-serialized events between the game engine and the main thread's Vue application.

## Requirements

### Requirement: WASM Initialization

The worker SHALL load and initialize the WASM module, then post a Ready event to the main thread.

#### Scenario: Successful startup
- **GIVEN** the worker script is loaded
- **WHEN** WASM initialization completes
- **THEN** the worker calls `init()`, posts a Ready event, posts the initial WorldSnapshot, and starts the game loop

#### Scenario: Initialization failure
- **GIVEN** WASM fails to load
- **WHEN** an error occurs during init
- **THEN** the worker posts an Error event with the failure message

### Requirement: Game Loop

The worker SHALL run a tick loop at 20 ticks/sec (50ms interval) with adjustable speed multiplier.

#### Scenario: Normal tick
- **GIVEN** the game loop is running at speed=1
- **WHEN** an interval fires
- **THEN** `tick(0.05)` is called and all outgoing events are posted to the main thread

#### Scenario: Speed multiplier
- **GIVEN** the speed is set to 3
- **WHEN** an interval fires
- **THEN** `tick(0.15)` is called (dt × speedMultiplier)

### Requirement: Event Forwarding

The worker SHALL relay IncomingEvents from the main thread to WASM and handle Pause/Resume/SetSpeed locally.

#### Scenario: PlaceBuilding forwarding
- **GIVEN** a PlaceBuilding message is received
- **WHEN** the worker processes it
- **THEN** it calls `push_event(JSON.stringify(event))` on the WASM module

#### Scenario: Pause/Resume
- **GIVEN** a Pause message is received
- **WHEN** the worker processes it
- **THEN** the game loop interval is cleared; on Resume, the interval is restarted

#### Scenario: SetSpeed
- **GIVEN** a SetSpeed message with speed=2
- **WHEN** the worker processes it
- **THEN** the speed multiplier is updated to 2 for subsequent ticks

### Requirement: Main Thread Bridge

The main thread SHALL provide a GameBridge class and Vue composable for communicating with the worker.

#### Scenario: GameBridge event dispatch
- **GIVEN** the worker posts an OutgoingEvent
- **WHEN** the main thread receives it
- **THEN** all registered callbacks are invoked with the event

#### Scenario: Vue composable reactive state
- **GIVEN** a Vue component uses `useGameBridge()`
- **WHEN** a WorldSnapshot is received
- **THEN** the `snapshot` shallowRef is updated and the component re-renders

#### Scenario: Bridge API
- **GIVEN** a Vue component calls `placeBuilding('Wall', 5, 10)`
- **WHEN** the bridge sends the message
- **THEN** a PlaceBuilding IncomingEvent is posted to the worker
