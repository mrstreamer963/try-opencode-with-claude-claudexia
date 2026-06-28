use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::world::World;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Node {
    x: i32,
    y: i32,
    cost: i32,    // g-cost
    priority: i32, // f-cost (g + h)
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority) // min-heap
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn heuristic(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x1 - x2).abs() + (y1 - y2).abs() // Manhattan distance
}

/// Find a path from (sx, sy) to (tx, ty) on the world grid.
/// Returns None if no path exists.
/// The returned path does NOT include the start position.
pub fn find_path(
    world: &World,
    sx: i32,
    sy: i32,
    tx: i32,
    ty: i32,
    occupied: &dyn Fn(i32, i32) -> bool,
) -> Option<Vec<(i32, i32)>> {
    if sx == tx && sy == ty {
        return Some(vec![]);
    }

    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut g_score: HashMap<(i32, i32), i32> = HashMap::new();

    g_score.insert((sx, sy), 0);
    open.push(Node {
        x: sx,
        y: sy,
        cost: 0,
        priority: heuristic(sx, sy, tx, ty),
    });

    let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    while let Some(current) = open.pop() {
        if current.x == tx && current.y == ty {
            // Reconstruct path
            let mut path = Vec::new();
            let mut pos = (tx, ty);
            while pos != (sx, sy) {
                path.push(pos);
                pos = came_from[&pos];
            }
            path.reverse();
            return Some(path);
        }

        let current_g = *g_score.get(&(current.x, current.y)).unwrap_or(&i32::MAX);
        if current.cost > current_g {
            continue; // stale entry
        }

        for (dx, dy) in &directions {
            let nx = current.x + dx;
            let ny = current.y + dy;

            // Target tile itself can be walked to even if "occupied" by destination
            let is_target = nx == tx && ny == ty;

            if !world.in_bounds(nx, ny) || !world.is_walkable(nx, ny) {
                continue;
            }
            if !is_target && occupied(nx, ny) {
                continue; // walls block
            }

            let new_g = current_g + 1;
            let existing_g = *g_score.get(&(nx, ny)).unwrap_or(&i32::MAX);

            if new_g < existing_g {
                g_score.insert((nx, ny), new_g);
                came_from.insert((nx, ny), (current.x, current.y));
                open.push(Node {
                    x: nx,
                    y: ny,
                    cost: new_g,
                    priority: new_g + heuristic(nx, ny, tx, ty),
                });
            }
        }
    }

    None // no path found
}

/// Find a path to a tile adjacent to the target (for interacting with buildings).
/// Returns path to the closest adjacent walkable tile, or None if unreachable.
pub fn find_path_adjacent(
    world: &World,
    sx: i32,
    sy: i32,
    tx: i32,
    ty: i32,
    occupied: &dyn Fn(i32, i32) -> bool,
) -> Option<Vec<(i32, i32)>> {
    // If already adjacent, return empty path
    let dx = (sx - tx).abs();
    let dy = (sy - ty).abs();
    if dx + dy == 1 {
        return Some(vec![]);
    }

    // Try each adjacent tile and pick shortest path
    let adj = [(0, -1), (0, 1), (-1, 0), (1, 0)];
    let mut best_path: Option<Vec<(i32, i32)>> = None;

    for (adx, ady) in &adj {
        let ax = tx + adx;
        let ay = ty + ady;
        if !world.in_bounds(ax, ay) || !world.is_walkable(ax, ay) || occupied(ax, ay) {
            continue;
        }
        if let Some(path) = find_path(world, sx, sy, ax, ay, occupied) {
            if best_path.as_ref().is_none_or(|bp| path.len() < bp.len()) {
                best_path = Some(path);
            }
        }
    }

    best_path
}
