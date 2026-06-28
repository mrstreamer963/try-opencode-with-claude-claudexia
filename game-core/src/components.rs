use bevy_ecs::prelude::*;

use crate::events::{BuildingType, ColonistTask};

#[derive(Component, Debug, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Debug, Clone)]
pub struct Needs {
    pub food: f32,
    pub sleep: f32,
}

#[derive(Component, Debug, Clone)]
pub struct ColonistTag {
    pub id: u32,
    pub name: String,
}

#[derive(Component, Debug, Clone)]
pub struct Building {
    pub id: u32,
    pub building_type: BuildingType,
}

#[derive(Component, Debug, Clone)]
pub struct CurrentTask {
    pub task: ColonistTask,
    pub target: Option<(i32, i32)>,
}

impl Default for CurrentTask {
    fn default() -> Self {
        Self {
            task: ColonistTask::Idle,
            target: None,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Path {
    pub steps: Vec<(i32, i32)>,
    pub current_step: usize,
}
