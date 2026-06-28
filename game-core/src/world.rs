use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::events::TileType;

pub const WIDTH: usize = 50;
pub const HEIGHT: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub tiles: Vec<TileType>,
    pub width: usize,
    pub height: usize,
}

impl World {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let mut tiles = Vec::with_capacity(WIDTH * HEIGHT);

        // Generate random tiles with weighted distribution
        for _ in 0..(WIDTH * HEIGHT) {
            let r: f32 = rng.random();
            let tile = if r < 0.15 {
                TileType::Water
            } else if r < 0.35 {
                TileType::Sand
            } else {
                TileType::Grass
            };
            tiles.push(tile);
        }

        let mut world = World {
            tiles,
            width: WIDTH,
            height: HEIGHT,
        };

        // Ensure connected walkable area via flood fill validation
        // If the largest connected component is too small, regenerate water tiles
        world.ensure_connected_walkable(&mut rng);
        world
    }

    fn ensure_connected_walkable(&mut self, rng: &mut impl Rng) {
        loop {
            let visited = self.find_largest_walkable_component();
            let walkable_count = self.tiles.iter().filter(|t| **t != TileType::Water).count();
            let visited_count = visited.iter().filter(|&&v| v).count();

            // If 90%+ of walkable tiles are in the largest component, we're good
            if walkable_count > 0 && visited_count as f32 / walkable_count as f32 >= 0.9 {
                // Convert disconnected walkable tiles to water or reconnect them
                for i in 0..self.tiles.len() {
                    if self.tiles[i] != TileType::Water && !visited[i] {
                        // Convert isolated tiles to match the main land
                        self.tiles[i] = if rng.random::<f32>() < 0.5 {
                            TileType::Water
                        } else {
                            TileType::Sand
                        };
                    }
                }
                break;
            }

            // Too fragmented — reduce water and retry
            for tile in &mut self.tiles {
                if *tile == TileType::Water && rng.random::<f32>() < 0.5 {
                    *tile = TileType::Grass;
                }
            }
        }
    }

    fn find_largest_walkable_component(&self) -> Vec<bool> {
        let total = self.width * self.height;
        let mut best_visited = vec![false; total];
        let mut global_visited = vec![false; total];
        let mut best_count = 0;

        for start in 0..total {
            if global_visited[start] || self.tiles[start] == TileType::Water {
                continue;
            }

            let mut visited = vec![false; total];
            let mut queue = VecDeque::new();
            queue.push_back(start);
            visited[start] = true;
            global_visited[start] = true;
            let mut count = 1usize;

            while let Some(idx) = queue.pop_front() {
                let x = (idx % self.width) as i32;
                let y = (idx / self.width) as i32;

                for (dx, dy) in &[(0, -1), (0, 1), (-1, 0), (1, 0)] {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                        let nidx = (ny as usize) * self.width + (nx as usize);
                        if !visited[nidx] && self.tiles[nidx] != TileType::Water {
                            visited[nidx] = true;
                            global_visited[nidx] = true;
                            queue.push_back(nidx);
                            count += 1;
                        }
                    }
                }
            }

            if count > best_count {
                best_count = count;
                best_visited = visited;
            }
        }

        best_visited
    }

    #[inline]
    pub fn idx(&self, x: i32, y: i32) -> usize {
        (y as usize) * self.width + (x as usize)
    }

    pub fn get_tile(&self, x: i32, y: i32) -> TileType {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return TileType::Water; // out of bounds = impassable
        }
        self.tiles[self.idx(x, y)]
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: TileType) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = self.idx(x, y);
            self.tiles[idx] = tile;
        }
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        let tile = self.get_tile(x, y);
        tile != TileType::Water
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32
    }
}
