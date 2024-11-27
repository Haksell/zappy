use crate::position::Position;
use crate::PlayerError::NoPlaceAvailable;
use crate::ZappyError;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Team {
    name: String,
    members: HashSet<u16>,
    spawn_positions: VecDeque<Position>,
}

impl Team {
    pub fn new(name: String, spawn_positions: VecDeque<Position>) -> Self {
        Self {
            name,
            members: HashSet::with_capacity(spawn_positions.len()),
            spawn_positions,
        }
    }

    pub fn add_member(&mut self, member_id: u16) -> Result<Position, ZappyError> {
        let pos = self
            .spawn_positions
            .pop_front()
            .ok_or_else(|| ZappyError::Player(NoPlaceAvailable(member_id, self.name.clone())))?;
        self.members.insert(member_id);
        Ok(pos)
    }

    pub fn remove_member(&mut self, member: u16) {
        self.members.remove(&member);
    }

    pub fn remaining_members(&self) -> u16 {
        self.spawn_positions.len() as u16
    }

    pub fn members_count(&self) -> usize {
        self.members.len()
    }

    pub fn has_member(&self, member_id: &u16) -> bool {
        self.members.contains(member_id)
    }

    pub fn add_next_spawn_position(&mut self, position: Position) {
        // TODO: max span_positions.len()?
        self.spawn_positions.push_back(position);
    }
}
