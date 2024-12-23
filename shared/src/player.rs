use crate::commands::PlayerCmd;
use crate::position::{Position, Side};
use crate::resource::{Stone, StoneSet};
use crate::{resource::Resource, GameError, DECREASED_HP_PER_FRAME, MAX_COMMANDS, MAX_PLAYER_LVL};
use crate::{LIFE_TICKS, LIVES_START};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

#[derive(Getters, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Player {
    team: String,
    id: u16,
    next_frame: u64,
    commands: VecDeque<PlayerCmd>,
    position: Position,
    inventory: [usize; Stone::SIZE],
    level: u8,
    remaining_life: u64,
    is_performing_incantation: bool,
}

impl Player {
    const LEVEL_REQUIREMENTS: [(StoneSet, usize); 7] = [
        //D  L  M  P  S  T
        ([0, 1, 0, 0, 0, 0], 1),
        ([1, 1, 0, 0, 1, 0], 2),
        ([0, 2, 0, 2, 1, 0], 2),
        ([1, 1, 0, 1, 2, 0], 4),
        ([2, 1, 3, 0, 1, 0], 4),
        ([2, 1, 0, 1, 3, 0], 6),
        ([2, 2, 2, 2, 2, 1], 6),
    ];

    pub fn new(id: u16, team: String, position: Position) -> Self {
        Self {
            id,
            team,
            next_frame: 0,
            commands: VecDeque::with_capacity(MAX_COMMANDS),
            position,
            inventory: [0; Stone::SIZE],
            level: 1,
            remaining_life: LIFE_TICKS * LIVES_START,
            is_performing_incantation: false,
        }
    }

    pub fn turn(&mut self, side: Side) {
        self.position.dir = self.position.dir.turn(side);
    }

    pub fn set_x(&mut self, x: usize) {
        self.position.x = x;
    }

    pub fn set_y(&mut self, y: usize) {
        self.position.y = y;
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    pub fn nxt_lvl_stone_requirements(&self) -> &'static StoneSet {
        &Self::LEVEL_REQUIREMENTS[self.level as usize - 1].0
    }

    pub fn nxt_lvl_player_cnt_requirements(&self) -> usize {
        Self::LEVEL_REQUIREMENTS[self.level as usize - 1].1
    }

    pub fn pop_command_from_queue(&mut self) -> Option<PlayerCmd> {
        // not an Option?
        self.commands.pop_front()
    }

    pub fn push_command_to_queue(&mut self, command: PlayerCmd) {
        self.commands.push_back(command);
    }

    pub fn set_next_frame(&mut self, value: u64) {
        self.next_frame = value;
    }

    pub fn add_to_inventory(&mut self, resource: Resource) {
        match resource {
            Resource::Stone(stone) => {
                self.inventory[stone as usize] += 1;
            }
            Resource::Nourriture => {
                self.remaining_life += LIFE_TICKS;
            }
        }
    }

    pub fn remove_from_inventory(&mut self, resource: Resource) -> bool {
        match resource {
            Resource::Stone(stone) => {
                if self.inventory[stone as usize] >= 1 {
                    self.inventory[stone as usize] -= 1;
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

    pub fn decrease_life(&mut self) {
        self.remaining_life -= DECREASED_HP_PER_FRAME;
    }

    pub fn start_incantation(&mut self) {
        self.is_performing_incantation = true;
    }

    pub fn stop_incantation(&mut self) -> Result<u8, GameError> {
        if self.level == MAX_PLAYER_LVL {
            Err(GameError::IncreasingLevelButIsAlreadyMax(self.id))
        } else if !self.is_performing_incantation {
            Err(GameError::IncreasingLevelWithNoIncantations(self.id))
        } else {
            self.is_performing_incantation = false;
            self.level += 1;
            Ok(self.level)
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
