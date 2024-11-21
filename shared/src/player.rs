use crate::TechnicalError::ConnectionCorrupted;
use crate::ZappyError::Technical;
use crate::{resource::Resource, PlayerCommand, ServerCommandToClient, ZappyError, MAX_COMMANDS};
use derive_getters::Getters;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

pub enum Side {
    Left,
    Right,
}

impl Direction {
    pub fn random() -> Self {
        static DIRECTIONS: [Direction; 4] = [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ];

        let mut rng = thread_rng();
        *DIRECTIONS.choose(&mut rng).unwrap()
    }

    pub fn turn(&self, side: Side) -> Self {
        match side {
            Side::Left => match self {
                Direction::North => Direction::West,
                Direction::West => Direction::South,
                Direction::South => Direction::East,
                Direction::East => Direction::North,
            },
            Side::Right => match self {
                Direction::North => Direction::East,
                Direction::East => Direction::South,
                Direction::South => Direction::West,
                Direction::West => Direction::North,
            },
        }
    }

    pub fn dx_dy(&self) -> (isize, isize) {
        match self {
            Direction::North => (0, -1),
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
    pub direction: Direction,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({}, {}, {:?})", self.x, self.y, self.direction)
    }
}

#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    team: String,
    id: u16,
    next_frame: u64,
    commands: VecDeque<PlayerCommand>,
    pub(crate) position: Position,
    inventory: [usize; Resource::SIZE],
    level: u8,
}

impl Player {
    pub fn new(
        id: u16,
        team: String,
        position: Position,
    ) -> Self {
        Self {
            id,
            team,
            next_frame: 0,
            commands: VecDeque::with_capacity(MAX_COMMANDS),
            position,
            inventory: [0; Resource::SIZE],
            level: 1,
        }
    }

    pub fn turn(&mut self, side: Side) {
        self.position.direction = self.position.direction.turn(side);
    }

    pub fn set_x(&mut self, x: usize) {
        self.position.x = x;
    }

    pub fn set_y(&mut self, y: usize) {
        self.position.y = y;
    }

    pub fn level_up(&mut self) {
        self.level += 1
    }


    pub fn pop_command_from_queue(&mut self) -> Option<PlayerCommand> {
        // not an Option?
        self.commands.pop_front()
    }

    pub fn push_command_to_queue(&mut self, command: PlayerCommand) {
        self.commands.push_back(command);
    }

    pub fn set_next_frame(&mut self, value: u64) {
        self.next_frame = value;
    }

    pub fn add_to_inventory(&mut self, resource: Resource) {
        self.inventory[resource as usize] += 1;
    }

    pub fn remove_from_inventory(&mut self, resource: Resource) -> bool {
        if self.inventory[resource as usize] >= 1 {
            self.inventory[resource as usize] -= 1;
            true
        } else {
            false
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Player(team: {}, id: {}, position: {}, inventory: {:?}, level: {})",
            self.team, self.id, self.position, self.inventory, self.level
        )
    }
}

impl Hash for Player {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.inventory.hash(state);
        self.level.hash(state);
    }
}
