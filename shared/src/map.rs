use crate::position::{Direction, Position};
use crate::resource::Resource;
use derive_getters::Getters;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

//TODO: change fields to private?
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    pub players: BTreeSet<u16>,
    pub resources: [usize; Resource::SIZE],
    pub eggs: BTreeMap<String, (usize, usize)>,
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
            players: BTreeSet::new(),
            resources: [0; Resource::SIZE],
            eggs: BTreeMap::new(),
        }
    }

    pub fn add_resource(&mut self, resource: Resource) {
        self.resources[usize::try_from(resource).unwrap()] += 1;
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

    pub fn add_player(&mut self, id: u16, team_name: &str, position: &Position) {
        log::debug!("Adding {} to the game field.", id);
        let cell = &mut self.field[position.y][position.x];
        cell.players.insert(id);
        cell.eggs.get_mut(team_name).unwrap().1 -= 1;
    }

    pub fn remove_player(&mut self, id: &u16, position: &Position) {
        log::debug!("Removing {} from the game field.", id);
        self.field[position.y][position.x].players.remove(id);
    }

    pub fn find_broadcast_source(&self, sender_pos: &Position, receiver_pos: &Position) -> u8 {
        let (width, height, receiver_x, receiver_y, sender_x, sender_y) = (
            *self.width() as isize,
            *self.height() as isize,
            receiver_pos.x as isize,
            receiver_pos.y as isize,
            sender_pos.x as isize,
            sender_pos.y as isize,
        );

        let north = (receiver_y - sender_y).rem_euclid(height) as usize;
        let east = (sender_x - receiver_x).rem_euclid(width) as usize;

        if north == 0 && east == 0 {
            return 0;
        }

        let south = (sender_y - receiver_y).rem_euclid(height) as usize;
        let west = (receiver_x - sender_x).rem_euclid(width) as usize;

        let (from_north, dy) = if north <= south {
            (true, north)
        } else {
            (false, south)
        };
        let (from_east, dx) = if east <= west {
            (true, east)
        } else {
            (false, west)
        };

        match dx.cmp(&dy) {
            std::cmp::Ordering::Less => {
                if from_north {
                    3
                } else {
                    7
                }
            }
            std::cmp::Ordering::Equal => match (from_north, from_east) {
                (true, true) => 2,
                (true, false) => 4,
                (false, false) => 6,
                (false, true) => 8,
            },
            std::cmp::Ordering::Greater => {
                if from_east {
                    1
                } else {
                    5
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: usize, y: usize) -> Position {
        Position {
            x,
            y,
            direction: Direction::random(),
        }
    }

    #[test]
    fn test_broadcast_source() {
        let map = Map::new(5, 5);
        let center = pos(2, 2);
        let expected = vec![
            vec![4, 3, 3, 3, 2],
            vec![5, 4, 3, 2, 1],
            vec![5, 5, 0, 1, 1],
            vec![5, 6, 7, 8, 1],
            vec![6, 7, 7, 7, 8],
        ];
        for y in 0..5 {
            for x in 0..5 {
                assert_eq!(
                    map.find_broadcast_source(&pos(x, y), &center),
                    expected[y][x]
                );
            }
        }
    }
}
