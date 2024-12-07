use crate::resource::{Resource, Stone, StoneSet};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::array;
use std::collections::{BTreeMap, VecDeque};
use std::f32::consts::TAU;

// TODO: change fields to private?
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Cell {
    pub players: BTreeMap<u16, CellPos>,
    pub stones: [VecDeque<CellPos>; Stone::SIZE],
    pub nourriture: VecDeque<CellPos>,
    pub eggs: BTreeMap<String, (usize, usize)>,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            players: BTreeMap::new(),
            stones: array::from_fn(|_| VecDeque::new()),
            eggs: BTreeMap::new(),
            nourriture: VecDeque::new(),
        }
    }

    pub fn random_position(&self) -> CellPos {
        // CellPos::random_spaced()
        // self.players.iter().map(f)
        CellPos::random() // TODO: random_spaced
    }

    pub fn add_resource(&mut self, resource: Resource) {
        let pos = CellPos::random();
        match resource {
            Resource::Stone(stone) => self.stones[stone.index()].push_back(pos),
            Resource::Nourriture => self.nourriture.push_back(pos),
        }
    }

    pub fn remove_resource(&mut self, resource: &Resource) -> bool {
        match resource {
            Resource::Stone(stone) => self.stones[stone.index()].pop_front().is_some(),
            Resource::Nourriture => self.nourriture.pop_front().is_some(),
        }
    }

    pub fn get_resources_copy(&self) -> Vec<Resource> {
        let capacity = self.nourriture.len() + self.stones.iter().map(VecDeque::len).sum::<usize>();
        let mut res = Vec::with_capacity(capacity);
        res.extend(std::iter::repeat(Resource::Nourriture).take(self.nourriture.len()));
        for (stone_idx, positions) in self.stones.iter().enumerate() {
            let stone = Resource::try_from(stone_idx).unwrap();
            res.extend(std::iter::repeat(stone).take(positions.len()));
        }

        res
    }

    pub fn reduce_current_from(&mut self, stone_set: &StoneSet) -> bool {
        let has_enough_resources = self
            .stones
            .iter()
            .zip(stone_set.iter())
            .all(|(a, &b)| a.len() >= b);
        if has_enough_resources {
            for (idx, &count) in stone_set.iter().enumerate() {
                for _ in 0..count {
                    self.stones[idx].pop_front();
                }
            }
        }
        has_enough_resources
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CellPos {
    pub x: f32,
    pub y: f32,
    pub angle: f32, // TODO: use
}

impl CellPos {
    pub fn random() -> Self {
        const PADDING: f32 = 0.08;
        let mut thread_rng = rand::thread_rng();
        Self {
            x: thread_rng.gen_range(PADDING..=1. - PADDING),
            y: thread_rng.gen_range(PADDING..=1. - PADDING),
            angle: thread_rng.gen_range(0.0..TAU),
        }
    }

    fn dist_squared(&self, other: &Self) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }

    fn random_spaced<I: Iterator<Item = Self>>(others: &mut I) -> Self {
        let mut max_dist_squared = 1.0;
        loop {
            let pos = Self::random();
            if others.all(|other| other.dist_squared(&pos) <= max_dist_squared) {
                return pos;
            }
            max_dist_squared *= 0.99;
        }
    }
}
