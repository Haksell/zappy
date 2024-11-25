use crate::position::{Position, Side};
use crate::resource::Mining;
use crate::{resource::Resource, PlayerCommand, MAX_COMMANDS, MAX_PLAYER_LVL};
use crate::{LIFE_TICKS, LIVES_START};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

#[derive(Getters, Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    team: String,
    id: u16,
    next_frame: u64,
    commands: VecDeque<PlayerCommand>,
    position: Position,
    inventory: [usize; Mining::SIZE],
    level: u8,
    remaining_life: u64,
}

impl Player {
    const LEVEL_RESOURCES_MASKS: [[usize; Mining::SIZE]; 7] = [
        // D  L  M  P  S  T
        [0, 1, 0, 0, 0, 0],
        [1, 1, 0, 0, 1, 0],
        [0, 2, 0, 2, 1, 0],
        [1, 1, 0, 1, 2, 0],
        [2, 1, 3, 0, 1, 0],
        [2, 1, 0, 1, 3, 0],
        [2, 2, 2, 2, 2, 1],
    ];

    pub fn new(id: u16, team: String, position: Position) -> Self {
        Self {
            id,
            team,
            next_frame: 0,
            commands: VecDeque::with_capacity(MAX_COMMANDS),
            position,
            inventory: [0; Mining::SIZE],
            level: 1,
            remaining_life: LIFE_TICKS * LIVES_START,
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

    fn get_right_resource_mask(level: u8) -> &'static [usize; Mining::SIZE] {
        &Self::LEVEL_RESOURCES_MASKS[level as usize - 1]
    }

    // TODO: need to test
    pub fn level_up(&mut self) -> bool {
        if self.level == MAX_PLAYER_LVL {
            log::error!(
                "Trying to level up {}, but the max level is already reached",
                self.id
            );
            return false;
        }
        let current_level_resource_mask = Player::get_right_resource_mask(self.level);
        let has_enough_resources = self
            .inventory
            .iter()
            .zip(current_level_resource_mask.iter())
            .all(|(a, b)| a >= b);
        if !has_enough_resources {
            false
        } else {
            for (idx, count) in current_level_resource_mask.iter().enumerate() {
                self.inventory[idx] -= count;
            }
            self.level += 1;
            true
        }
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
        match resource {
            Resource::Mining(mining) => {
                self.inventory[mining as usize] += 1;
            }
            Resource::Nourriture => {
                self.remaining_life += LIFE_TICKS;
            }
        }
    }

    pub fn remove_from_inventory(&mut self, resource: Resource) -> bool {
        match resource {
            Resource::Mining(mining) => {
                if self.inventory[mining as usize] >= 1 {
                    self.inventory[mining as usize] -= 1;
                    true
                } else {
                    false
                }
            }
            Resource::Nourriture => match self.remaining_life.checked_sub(LIFE_TICKS) {
                Some(remaining_life) => {
                    self.remaining_life = remaining_life;
                    true
                }
                None => false,
            },
        }
    }

    pub fn decrement_life(&mut self) {
        self.remaining_life -= 1;
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
