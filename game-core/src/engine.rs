use bevy_ecs::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::events::*;
use crate::pathfinding;
use crate::world::{World, move_cost};

const FOOD_DECAY_RATE: f32 = 0.02;
const SLEEP_DECAY_RATE: f32 = 0.015;
const NEED_THRESHOLD: f32 = 0.5;
const RESTORE_RATE: f32 = 0.1;
const BASE_MOVE_SPEED: f32 = 3.0;
const COLONIST_NAMES: &[&str] = &["Ada", "Rex", "Nova", "Colt", "Iris", "Juno", "Axel", "Wren"];

pub struct GameEngine {
    pub ecs_world: bevy_ecs::world::World,
    pub tile_world: World,
    pub incoming_events: Vec<IncomingEvent>,
    pub outgoing_events: Vec<OutgoingEvent>,
    next_entity_id: u32,
    tiles_dirty: bool,
}

impl GameEngine {
    pub fn new() -> Self {
        let tile_world = World::new();
        let ecs_world = bevy_ecs::world::World::new();

        let mut engine = GameEngine {
            ecs_world,
            tile_world,
            incoming_events: Vec::new(),
            outgoing_events: Vec::new(),
            next_entity_id: 1,
            tiles_dirty: true,
        };

        engine.spawn_colonists(3);
        engine
    }

    fn next_id(&mut self) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        id
    }

    // ── Task 4.2: Colonist spawning ──
    fn spawn_colonists(&mut self, count: usize) {
        let mut rng = rand::rng();
        let mut names_used = 0;

        for _ in 0..count {
            // Find a random walkable position
            let (x, y) = loop {
                let x = rng.random_range(0..self.tile_world.width as i32);
                let y = rng.random_range(0..self.tile_world.height as i32);
                if self.tile_world.is_walkable(x, y) {
                    break (x, y);
                }
            };

            let id = self.next_id();
            let name = COLONIST_NAMES[names_used % COLONIST_NAMES.len()].to_string();
            names_used += 1;

            self.ecs_world.spawn((
                ColonistTag { id, name },
                Position { x, y },
                Needs {
                    food: 1.0,
                    sleep: 1.0,
                },
                CurrentTask::default(),
            ));
        }
    }

    // ── Task 4.3: Building placement ──
    fn place_building(&mut self, building_type: BuildingType, x: i32, y: i32) {
        // Validate tile
        if !self.tile_world.in_bounds(x, y) {
            self.outgoing_events.push(OutgoingEvent::Error {
                message: format!("Position ({}, {}) is out of bounds", x, y),
            });
            return;
        }

        if !self.tile_world.is_walkable(x, y) {
            self.outgoing_events.push(OutgoingEvent::Error {
                message: format!("Cannot place building on water at ({}, {})", x, y),
            });
            return;
        }

        // Check if occupied by another building
        let mut occupied = false;
        let mut query = self.ecs_world.query::<(&Building, &Position)>();
        for (_, pos) in query.iter(&self.ecs_world) {
            if pos.x == x && pos.y == y {
                occupied = true;
                break;
            }
        }

        if occupied {
            self.outgoing_events.push(OutgoingEvent::Error {
                message: format!("Tile ({}, {}) is already occupied by a building", x, y),
            });
            return;
        }

        let id = self.next_id();
        self.ecs_world
            .spawn((Building { id, building_type }, Position { x, y }));

        self.outgoing_events.push(OutgoingEvent::BuildingPlaced {
            building_type,
            x,
            y,
        });
    }

    // ── Task 7.2: Push incoming event ──
    pub fn push_event(&mut self, json: &str) {
        match serde_json::from_str::<IncomingEvent>(json) {
            Ok(event) => self.incoming_events.push(event),
            Err(e) => {
                self.outgoing_events.push(OutgoingEvent::Error {
                    message: format!("Failed to parse event: {}", e),
                });
            }
        }
    }

    // ── Task 7.3: Drain outgoing events ──
    pub fn drain_events(&mut self) -> String {
        let events: Vec<OutgoingEvent> = self.outgoing_events.drain(..).collect();
        serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string())
    }

    // ── Task 7.1: Main tick function ──
    pub fn tick(&mut self, dt: f32) {
        // 1. Process incoming events
        let events: Vec<IncomingEvent> = self.incoming_events.drain(..).collect();
        for event in events {
            match event {
                IncomingEvent::PlaceBuilding {
                    building_type,
                    x,
                    y,
                } => {
                    self.place_building(building_type, x, y);
                }
                // SetSpeed, Pause, Resume are handled by the worker, not the engine
                IncomingEvent::SetSpeed { .. } | IncomingEvent::Pause | IncomingEvent::Resume => {}
            }
        }

        // 2. Decay needs
        self.decay_needs(dt);

        // 3. Assign tasks to idle colonists
        self.assign_tasks();

        // 4. Move colonists along paths
        self.move_colonists(dt);

        // 5. Process active tasks (eating, sleeping)
        self.process_tasks(dt);

        // 6. Emit world snapshot
        self.emit_snapshot();
    }

    // ── Task 6.1: Needs decay ──
    fn decay_needs(&mut self, dt: f32) {
        let mut query = self.ecs_world.query::<&mut Needs>();
        for mut needs in query.iter_mut(&mut self.ecs_world) {
            needs.food = (needs.food - FOOD_DECAY_RATE * dt).max(0.0);
            needs.sleep = (needs.sleep - SLEEP_DECAY_RATE * dt).max(0.0);
        }
    }

    // ── Task 6.2: Task assignment ──
    fn assign_tasks(&mut self) {
        // Collect building positions first
        let mut beds: Vec<(i32, i32)> = Vec::new();
        let mut bushes: Vec<(i32, i32)> = Vec::new();
        {
            let mut bq = self.ecs_world.query::<(&Building, &Position)>();
            for (building, pos) in bq.iter(&self.ecs_world) {
                match building.building_type {
                    BuildingType::Bed => beds.push((pos.x, pos.y)),
                    BuildingType::BerryBush => bushes.push((pos.x, pos.y)),
                    BuildingType::Wall => {}
                }
            }
        }

        // Collect wall positions for pathfinding
        let wall_positions: Vec<(i32, i32)> = {
            let mut wq = self.ecs_world.query::<(&Building, &Position)>();
            wq.iter(&self.ecs_world)
                .filter(|(b, _)| b.building_type == BuildingType::Wall)
                .map(|(_, p)| (p.x, p.y))
                .collect()
        };

        // Collect idle colonists that need tasks
        let mut assignments: Vec<(Entity, ColonistTask, Vec<(i32, i32)>, (i32, i32))> = Vec::new();

        {
            let mut query = self
                .ecs_world
                .query::<(Entity, &Position, &Needs, &CurrentTask)>();

            for (entity, pos, needs, task) in query.iter(&self.ecs_world) {
                if task.task != ColonistTask::Idle {
                    continue;
                }

                let occupied = |x: i32, y: i32| -> bool {
                    wall_positions.iter().any(|&(wx, wy)| wx == x && wy == y)
                };

                // Check lowest need
                if needs.food < NEED_THRESHOLD && !bushes.is_empty() {
                    // Find nearest berry bush
                    if let Some((bx, by, path)) =
                        find_nearest(&self.tile_world, pos.x, pos.y, &bushes, &occupied)
                    {
                        assignments.push((entity, ColonistTask::MovingToFood, path, (bx, by)));
                        continue;
                    }
                }

                if needs.sleep < NEED_THRESHOLD && !beds.is_empty() {
                    // Find nearest bed
                    if let Some((bx, by, path)) =
                        find_nearest(&self.tile_world, pos.x, pos.y, &beds, &occupied)
                    {
                        assignments.push((entity, ColonistTask::MovingToBed, path, (bx, by)));
                        continue;
                    }
                }

                // Idle wandering
                let mut rng = rand::rng();
                let wx = pos.x + rng.random_range(-3..=3);
                let wy = pos.y + rng.random_range(-3..=3);
                if self.tile_world.in_bounds(wx, wy) && self.tile_world.is_walkable(wx, wy) {
                    if let Some(path) =
                        pathfinding::find_path(&self.tile_world, pos.x, pos.y, wx, wy, &occupied)
                    {
                        if !path.is_empty() {
                            assignments.push((entity, ColonistTask::Wandering, path, (wx, wy)));
                        }
                    }
                }
            }
        }

        // Apply assignments
        for (entity, task_type, path, target) in assignments {
            if let Some(mut task) = self.ecs_world.get_mut::<CurrentTask>(entity) {
                task.task = task_type;
                task.target = Some(target);
            }
            if !path.is_empty() {
                self.ecs_world.entity_mut(entity).insert(Path {
                    steps: path,
                    current_step: 0,
                    move_progress: 0.0,
                });
            }
        }
    }

    // Move colonists along paths using rate-based progress with carry-the-remainder.
    // `move_progress` accumulates at `BASE_MOVE_SPEED / move_cost(target_tile)` cells/sec.
    // When it crosses 1.0, advance `current_step`, subtract 1.0, and re-check against the
    // next cell's cost so a single large `dt` can correctly traverse multiple cells.
    fn move_colonists(&mut self, dt: f32) {
        let mut completed: Vec<Entity> = Vec::new();
        let tile_world = &self.tile_world;

        {
            let mut query = self.ecs_world.query::<(Entity, &mut Position, &mut Path)>();
            for (entity, mut pos, mut path) in query.iter_mut(&mut self.ecs_world) {
                if path.current_step >= path.steps.len() {
                    completed.push(entity);
                    continue;
                }

                let mut remaining_dt = dt;
                // Advance through as many cells as `remaining_dt` allows.
                while remaining_dt > 0.0 && path.current_step < path.steps.len() {
                    let (nx, ny) = path.steps[path.current_step];
                    let cost = move_cost(tile_world.get_tile(nx, ny));
                    if !cost.is_finite() {
                        // Impassable target — freeze. Should not happen given pathfinding,
                        // but guard so we never teleport onto Water.
                        break;
                    }
                    let rate = BASE_MOVE_SPEED / cost; // cells/sec
                    let needed = 1.0 - path.move_progress; // progress to reach the next cell
                    let possible = remaining_dt * rate; // progress affordable with remaining_dt

                    if possible < needed {
                        // Stay in the current "from cell"; partial progress toward the next.
                        path.move_progress += possible;
                        remaining_dt = 0.0;
                    } else {
                        // Cross the boundary: spend just enough dt to finish this cell,
                        // advance Position to the entered cell, and carry the remainder.
                        let dt_used = needed / rate;
                        remaining_dt -= dt_used;
                        pos.x = nx;
                        pos.y = ny;
                        path.current_step += 1;
                        path.move_progress = 0.0;
                    }
                }

                if path.current_step >= path.steps.len() {
                    completed.push(entity);
                }
            }
        }

        for entity in completed {
            // Remove path component
            self.ecs_world.entity_mut(entity).remove::<Path>();

            // Transition task
            if let Some(mut task) = self.ecs_world.get_mut::<CurrentTask>(entity) {
                match task.task {
                    ColonistTask::MovingToFood => task.task = ColonistTask::Eating,
                    ColonistTask::MovingToBed => task.task = ColonistTask::Sleeping,
                    ColonistTask::Wandering => {
                        task.task = ColonistTask::Idle;
                        task.target = None;
                    }
                    _ => {}
                }
            }
        }
    }

    // ── Tasks 6.3, 6.4: Process eating/sleeping ──
    fn process_tasks(&mut self, dt: f32) {
        let mut finished: Vec<Entity> = Vec::new();

        {
            let mut query = self.ecs_world.query::<(Entity, &mut Needs, &CurrentTask)>();
            for (entity, mut needs, task) in query.iter_mut(&mut self.ecs_world) {
                match task.task {
                    ColonistTask::Eating => {
                        needs.food = (needs.food + RESTORE_RATE * dt).min(1.0);
                        if needs.food >= 0.95 {
                            finished.push(entity);
                        }
                    }
                    ColonistTask::Sleeping => {
                        needs.sleep = (needs.sleep + RESTORE_RATE * dt).min(1.0);
                        if needs.sleep >= 0.95 {
                            finished.push(entity);
                        }
                    }
                    _ => {}
                }
            }
        }

        for entity in finished {
            if let Some(mut task) = self.ecs_world.get_mut::<CurrentTask>(entity) {
                task.task = ColonistTask::Idle;
                task.target = None;
            }
        }
    }

    // ── Task 7.4: World snapshot serialization ──
    fn emit_snapshot(&mut self) {
        let mut colonists = Vec::new();
        {
            let mut query =
                self.ecs_world
                    .query::<(&ColonistTag, &Position, &Needs, &CurrentTask, Option<&Path>)>();
            for (tag, pos, needs, task, path) in query.iter(&self.ecs_world) {
                // For a moving colonist, expose the cell it is heading toward and the
                // current float progress so the renderer can lerp; stationary colonists
                // get nulls + 0.0 progress (no `Path` component).
                let (next_x, next_y, move_progress) = match path {
                    Some(p) if p.current_step < p.steps.len() => {
                        let (nx, ny) = p.steps[p.current_step];
                        (Some(nx), Some(ny), p.move_progress)
                    }
                    _ => (None, None, 0.0),
                };

                colonists.push(ColonistState {
                    id: tag.id,
                    name: tag.name.clone(),
                    x: pos.x,
                    y: pos.y,
                    next_x,
                    next_y,
                    move_progress,
                    food: needs.food,
                    sleep: needs.sleep,
                    task: task.task,
                });
            }
        }

        let mut buildings = Vec::new();
        {
            let mut query = self.ecs_world.query::<(&Building, &Position)>();
            for (building, pos) in query.iter(&self.ecs_world) {
                buildings.push(BuildingState {
                    id: building.id,
                    building_type: building.building_type,
                    x: pos.x,
                    y: pos.y,
                });
            }
        }

        let snapshot = WorldSnapshot {
            tiles: if self.tiles_dirty {
                self.tiles_dirty = false;
                self.tile_world.tiles.clone()
            } else {
                self.tile_world.tiles.clone() // always send for simplicity at this world size
            },
            width: self.tile_world.width,
            height: self.tile_world.height,
            colonists,
            buildings,
        };

        self.outgoing_events
            .push(OutgoingEvent::WorldSnapshot { snapshot });
    }

    /// Get initial world snapshot as JSON
    pub fn get_initial_snapshot(&mut self) -> String {
        self.emit_snapshot();
        self.drain_events()
    }
}

fn find_nearest(
    world: &World,
    sx: i32,
    sy: i32,
    targets: &[(i32, i32)],
    occupied: &dyn Fn(i32, i32) -> bool,
) -> Option<(i32, i32, Vec<(i32, i32)>)> {
    let mut best: Option<(i32, i32, Vec<(i32, i32)>)> = None;

    for &(tx, ty) in targets {
        if let Some(path) = pathfinding::find_path_adjacent(world, sx, sy, tx, ty, occupied) {
            if best.as_ref().is_none_or(|(_, _, bp)| path.len() < bp.len()) {
                best = Some((tx, ty, path));
            }
        }
    }

    best
}
