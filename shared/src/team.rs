use crate::LogicalError::MaxPlayersReached;
use crate::ZappyError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug)]
pub struct Team {
    members: HashSet<u16>,
    max_members: u16,
}

impl Team {
    pub fn new(max_members: u16) -> Self {
        Self {
            members: HashSet::with_capacity(max_members as usize),
            max_members,
        }
    }

    pub fn add_member(&mut self, member: u16) -> Result<(), ZappyError> {
        if self.members.len() < self.max_members as usize {
            self.members.insert(member);
            Ok(())
        } else {
            Err(ZappyError::Logical(MaxPlayersReached(
                member,
                self.remaining_members(),
            )))
        }
    }

    pub fn remove_member(&mut self, member: u16) {
        self.members.remove(&member);
    }

    pub fn remaining_members(&self) -> u16 {
        self.max_members - self.members.len() as u16
    }

    pub fn members_count(&self) -> usize {
        self.members.len()
    }

    pub fn increment_max_members(&mut self) {
        self.max_members += 1;
    }

    pub fn max_members(&self) -> u16 {
        self.max_members
    }
}
