// ── Tile Types ──
export enum TileType {
  Water = 'Water',
  Sand = 'Sand',
  Grass = 'Grass',
}

// ── Building Types ──
export enum BuildingType {
  Wall = 'Wall',
  Bed = 'Bed',
  BerryBush = 'BerryBush',
}

// ── Colonist Task ──
export enum ColonistTask {
  Idle = 'Idle',
  MovingToFood = 'MovingToFood',
  Eating = 'Eating',
  MovingToBed = 'MovingToBed',
  Sleeping = 'Sleeping',
  Wandering = 'Wandering',
}

// ── State Snapshots ──
export interface ColonistState {
  id: number
  name: string
  x: number
  y: number
  /** Cell being entered, or null when stationary (no active Path). */
  next_x: number | null
  next_y: number | null
  /** Fraction in [0, 1] from (x, y) toward (next_x, next_y); 0 when stationary. */
  move_progress: number
  food: number
  sleep: number
  task: ColonistTask
}

export interface BuildingState {
  id: number
  buildingType: BuildingType
  x: number
  y: number
}

export interface WorldSnapshot {
  tiles: TileType[]
  width: number
  height: number
  colonists: ColonistState[]
  buildings: BuildingState[]
}

// ── Incoming Events (main thread → worker → WASM) ──
export type IncomingEvent =
  | { type: 'PlaceBuilding'; buildingType: BuildingType; x: number; y: number }
  | { type: 'SetSpeed'; speed: number }
  | { type: 'Pause' }
  | { type: 'Resume' }

// ── Outgoing Events (WASM → worker → main thread) ──
export type OutgoingEvent =
  | { type: 'Ready' }
  | { type: 'WorldSnapshot'; snapshot: WorldSnapshot }
  | { type: 'BuildingPlaced'; buildingType: BuildingType; x: number; y: number }
  | { type: 'Error'; message: string }
