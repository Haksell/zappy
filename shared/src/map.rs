use crate::position::{Direction, Position};
use crate::resource::{Resource, Stone};
use derive_getters::Getters;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

//TODO: change fields to private?
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    pub players: BTreeSet<u16>,
    pub stones: [usize; Stone::SIZE],
    pub nourriture: usize,
    pub eggs: BTreeMap<String, (usize, usize)>,
}

//TODO: change fields to private?
#[derive(Debug, Serialize, Deserialize, Getters, Default)]
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
            stones: [0; Stone::SIZE],
            eggs: BTreeMap::new(),
            nourriture: 0,
        }
    }

    pub fn add_resource(&mut self, resource: Resource) {
        match resource {
            Resource::Stone(stone) => self.stones[usize::try_from(stone).unwrap()] += 1,
            Resource::Nourriture => self.nourriture += 1,
        }
    }

    pub fn remove_resource(&mut self, resource: &Resource) -> bool {
        match resource {
            Resource::Stone(stone) => {
                let stone_count = &mut self.stones[usize::try_from(*stone).unwrap()];
                if *stone_count > 0 {
                    *stone_count -= 1;
                    true
                } else {
                    false
                }
            }
            Resource::Nourriture => {
                if self.nourriture > 0 {
                    self.nourriture -= 1;
                    true
                } else {
                    false
                }
            }
        }
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

        let source_if_east = match dx.cmp(&dy) {
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
        };

        let dir_shift = match receiver_pos.direction {
            Direction::North => 6,
            Direction::East => 0,
            Direction::South => 2,
            Direction::West => 4,
        };

        ((source_if_east + dir_shift - 1) & 7) + 1
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
    fn test_broadcast_source_center() {
        let map = Map::new(5, 5);
        let receiver = Position {
            x: 2,
            y: 2,
            direction: Direction::East,
        };
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
                    map.find_broadcast_source(&pos(x, y), &receiver),
                    expected[y][x]
                );
            }
        }
    }

    #[test]
    fn test_broadcast_source_asymetric() {
        let map = Map::new(5, 5);
        let receiver = Position {
            x: 0,
            y: 1,
            direction: Direction::North,
        };
        let expected = vec![
            vec![1, 8, 7, 3, 2],
            vec![0, 7, 7, 3, 3],
            vec![5, 6, 7, 3, 4],
            vec![5, 5, 6, 4, 5],
            vec![1, 1, 8, 2, 1],
        ];
        for y in 0..5 {
            for x in 0..5 {
                assert_eq!(
                    map.find_broadcast_source(&pos(x, y), &receiver),
                    expected[y][x]
                );
            }
        }
    }
}
