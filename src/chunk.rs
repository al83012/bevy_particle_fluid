use bevy::{prelude::*, utils::HashSet};
use std::array;

pub const CHUNK_SIZE: usize = 64;

pub struct Chunk<T> {
    pub cells: [T; CHUNK_SIZE * CHUNK_SIZE],
}

#[derive(Resource)]
pub struct EntityLookupChunk(pub Chunk<HashSet<Entity>>);

#[derive(Component, Default, Debug, Clone)]
pub struct ChunkPosition {
    pub pos: UVec2,
}

impl<T> Chunk<T> {
    pub fn get_chunk_pos(x: f32, y: f32) -> Option<UVec2> {
        let pos =
            if x > -0.5 && x < CHUNK_SIZE as f32 + 0.5 && y > -0.5 && y < CHUNK_SIZE as f32 + 0.5 {
                UVec2::new(x.round() as u32, y.round() as u32)
            } else {
                return None;
            };

        Some(pos)
    }

    pub fn index(x: usize, y: usize) -> Option<usize> {
        if !Self::is_valid_pos(x, y) {
            return None;
        }
        Some(x + y * CHUNK_SIZE)
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        Some(&self.cells[Self::index(x, y)?])
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        Some(&mut self.cells[Self::index(x, y)?])
    }

    pub fn is_valid_pos(x: usize, y: usize) -> bool {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE {
            false
        } else {
            true
        }
    }

    pub fn get_neighbors(&self, x: usize, y: usize) -> [Option<&T>; 8] {
        let mut elements = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ]
        .into_iter()
        .map(|(dx, dy)| {
            let x_i = x as i32;
            let y_i = y as i32;
            if dx == -1 && x == 0 {
                return None;
            }
            if dx == 1 && x == CHUNK_SIZE - 1 {
                return None;
            }
            if dy == -1 && y == 0 {
                return None;
            }
            if dy == 1 && y == CHUNK_SIZE - 1 {
                return None;
            }
            self.get((x_i + dx) as usize, (y_i + dy) as usize)
        });

        [
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
        ]
    }

    pub fn get_neighborhood(&self, x: usize, y: usize) -> [Option<&T>; 9] {
        let mut elements = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 0),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ]
        .into_iter()
        .map(|(dx, dy)| {
            let x_i = x as i32;
            let y_i = y as i32;
            if dx == -1 && x == 0 {
                return None;
            }
            if dx == 1 && x == CHUNK_SIZE - 1 {
                return None;
            }
            if dy == -1 && y == 0 {
                return None;
            }
            if dy == 1 && y == CHUNK_SIZE - 1 {
                return None;
            }
            self.get((x_i + dx) as usize, (y_i + dy) as usize)
        });

        [
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
            elements.next().unwrap(),
        ]
    }
}

impl Default for EntityLookupChunk {
    fn default() -> Self {
        Self(Chunk {
            cells: array::from_fn(|_| HashSet::new()),
        })
    }
}

impl EntityLookupChunk {
    pub fn insert(&mut self, element: Entity, x: usize, y: usize) -> Option<()> {
        self.0.get_mut(x, y)?.insert(element);
        Some(())
    }

    pub fn get_neighborhood_entities(&self, x: usize, y: usize) -> Vec<Entity> {
        let neighborhood = self.0.get_neighborhood(x, y);

        let entities = neighborhood
            .as_slice()
            .iter()
            .filter_map(|x| *x)
            .flatten()
            .cloned()
            .collect::<Vec<_>>();

        entities
    }
}
