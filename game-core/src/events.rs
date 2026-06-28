use serde::{Deserialize, Serialize};

// ── Tile Types ──
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Water,
    Sand,
    Grass,
}

// ── Building Types ──
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType {
    Wall,
    Bed,
    BerryBush,
}

// ── Colonist Task ──
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColonistTask {
    Idle,
    MovingToFood,
    Eating,
    MovingToBed,
    Sleeping,
    Wandering,
}

// ── Incoming Events (from JS) ──
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum IncomingEvent {
    PlaceBuilding {
        #[serde(rename = "buildingType")]
        building_type: BuildingType,
        x: i32,
        y: i32,
    },
    SetSpeed {
        speed: f32,
    },
    Pause,
    Resume,
}

// ── Outgoing Events (to JS) ──
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum OutgoingEvent {
    Ready,
    WorldSnapshot {
        snapshot: WorldSnapshot,
    },
    BuildingPlaced {
        #[serde(rename = "buildingType")]
        building_type: BuildingType,
        x: i32,
        y: i32,
    },
    Error {
        message: String,
    },
}

// ── State Snapshots ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColonistState {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    /// Integer cell being entered, or `None` when stationary (no `Path`).
    pub next_x: Option<i32>,
    pub next_y: Option<i32>,
    /// Fraction of the way from `(x, y)` to `(next_x, next_y)` in [0, 1]. 0.0 when stationary.
    pub move_progress: f32,
    pub food: f32,
    pub sleep: f32,
    pub task: ColonistTask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingState {
    pub id: u32,
    #[serde(rename = "buildingType")]
    pub building_type: BuildingType,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub tiles: Vec<TileType>,
    pub width: usize,
    pub height: usize,
    pub colonists: Vec<ColonistState>,
    pub buildings: Vec<BuildingState>,
}
