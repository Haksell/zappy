use crate::player::{Direction, Position};
use crate::resource::Resource;
use crate::Egg;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use derive_getters::Getters;

//TODO: change fields to private?
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    pub players: HashSet<u16>,
    pub resources: [usize; Resource::SIZE],
    pub eggs: Vec<Egg>,
}

//TODO: change fields to private?
#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct Map {
    #[getter(skip)]
    pub field: Vec<Vec<Cell>>,
    width: usize,
    height: usize,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            players: HashSet::new(),
            resources: [0; Resource::SIZE],
            eggs: Vec::new(),
        }
    }

    pub fn add_resource(&mut self, resource: Resource) {
        self.resources[resource as usize] += 1;
    }
}

impl Map {
    // TODO: better procedural generation
    pub fn new(width: usize, height: usize) -> Self {
        let mut map = vec![vec![Cell::new(); width]; height];
        for y in 0..height {
            for x in 0..width {
                map[y][x].add_resource(Resource::random());
            }
        }
        Self {
            field: map,
            width,
            height,
        }
    }

    pub fn random_position(&self) -> Position {
        let mut thread_rng = rand::thread_rng();
        Position {
            x: thread_rng.gen_range(0..self.width),
            y: thread_rng.gen_range(0..self.height),
            direction: Direction::random(),
        }
    }

    pub fn add_player(&mut self, id: u16, position: &Position) {
        log::debug!("Adding {} to the game field.", id);
        self.field[position.y][position.x].players.insert(id);
    }

    pub fn remove_player(&mut self, id: &u16, position: &Position) {
        log::debug!("Removing {} from the game field.", id);
        self.field[position.y][position.x].players.remove(id);
    }
}
