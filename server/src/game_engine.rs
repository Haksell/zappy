use crate::args::ServerArgs;
use derive_getters::Getters;
use shared::resource::StoneSetOperations;
use shared::{
    commands::PlayerCommand,
    map::Map,
    player::Player,
    position::{Direction, Position, Side},
    resource::Resource,
    team::Team,
    Egg, ServerResponse,
    TechnicalError::IsNotConnectedToServer,
    ZappyError,
    ZappyError::Technical,
    MAX_COMMANDS,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
};

#[derive(Debug, Getters)]
pub struct GameEngine {
    teams: HashMap<String, Team>,
    players: HashMap<u16, Player>,
    eggs: HashMap<u64, Vec<Egg>>,
    incantation: HashMap<u64, Vec<u16>>,
    map: Map,
    frame: u64,
}

impl GameEngine {
    pub async fn from(args: &ServerArgs) -> Result<Self, Box<dyn Error>> {
        let mut map = Map::new(args.width, args.height);
        let teams = args
            .names
            .iter()
            .map(|team_name| {
                let spawn_positions: VecDeque<Position> =
                    (0..args.clients).map(|_| map.random_position()).collect();
                for pos in &spawn_positions {
                    map.field[pos.y][pos.x]
                        .eggs
                        .entry(team_name.clone())
                        .or_insert((0, 0))
                        .1 += 1;
                }
                (
                    team_name.clone(),
                    Team::new(team_name.clone(), spawn_positions),
                )
            })
            .collect();
        Ok(Self {
            incantation: HashMap::new(),
            teams,
            players: HashMap::new(),
            eggs: HashMap::new(),
            map,
            frame: 0,
        })
    }

    fn handle_move(&mut self, player_id: u16, direction: &Direction) {
        let player = self.players.get_mut(&player_id).unwrap();
        let current_x = player.position().x;
        let current_y = player.position().y;

        let (dx, dy) = direction.dx_dy();
        player.set_x((current_x as isize + dx).rem_euclid(*self.map.width() as isize) as usize);
        player.set_y((current_y as isize + dy).rem_euclid(*self.map.height() as isize) as usize);

        self.map.field[current_y][current_x]
            .players
            .remove(player.id());
        self.map.field[player.position().y][player.position().x]
            .players
            .insert(*player.id());
    }

    fn apply_cmd(&mut self, player_id: u16, command: &PlayerCommand) -> Vec<(u16, ServerResponse)> {
        log::debug!("Executing command: {:?} for {}", command, player_id);
        match command {
            PlayerCommand::Left => {
                let player = self.players.get_mut(&player_id).unwrap();
                player.turn(Side::Left);
                vec![(player_id, ServerResponse::Ok)]
            }
            PlayerCommand::Right => {
                let player = self.players.get_mut(&player_id).unwrap();
                player.turn(Side::Right);
                vec![(player_id, ServerResponse::Ok)]
            }
            PlayerCommand::Move => {
                let player_direction = {
                    let player = self.players.get_mut(&player_id).unwrap();
                    player.position().direction
                };
                self.handle_move(player_id, &player_direction);
                vec![(player_id, ServerResponse::Ok)]
            }
            PlayerCommand::Take { resource_name } => {
                let response = Resource::try_from(resource_name.as_str())
                    .map(|resource| {
                        let player = self.players.get_mut(&player_id).unwrap();
                        let cell = &mut self.map.field[player.position().y][player.position().x];
                        if cell.remove_resource(&resource) {
                            player.add_to_inventory(resource);
                            ServerResponse::Ok
                        } else {
                            ServerResponse::Ko
                        }
                    })
                    .unwrap_or(ServerResponse::Ko);
                vec![(player_id, response)]
            }
            PlayerCommand::Put { resource_name } => {
                let response = Resource::try_from(resource_name.as_str())
                    .map(|resource| {
                        let player = self.players.get_mut(&player_id).unwrap();
                        let cell = &mut self.map.field[player.position().y][player.position().x];
                        if player.remove_from_inventory(resource) {
                            cell.add_resource(resource);
                            ServerResponse::Ok
                        } else {
                            ServerResponse::Ko
                        }
                    })
                    .unwrap_or(ServerResponse::Ko);
                vec![(player_id, response)]
            }
            PlayerCommand::See => {
                let player = self.players.get(&player_id).unwrap();
                let pos = *player.position();
                let (player_x, player_y) = (pos.x as isize, pos.y as isize);
                let (width, height) = (self.map_width() as isize, self.map_height() as isize);
                let mut response = Vec::with_capacity((*player.level() as usize + 1).pow(2));
                for line in 0..=(*player.level() as isize) {
                    for idx in -line..=line {
                        let (x, y) = {
                            let (x, y) = match pos.direction {
                                Direction::North => (player_x + idx, player_y - line),
                                Direction::East => (player_x + line, player_y + idx),
                                Direction::South => (player_x - idx, player_y + line),
                                Direction::West => (player_x - line, player_y - idx),
                            };
                            (x.rem_euclid(width) as usize, y.rem_euclid(height) as usize)
                        };
                        let is_same_pos = x == player.position().x && y == player.position().y;
                        let cell = &self.map.field[y][x];
                        let mut cell_response =
                            vec!["player"; cell.players.len() - is_same_pos as usize];
                        cell_response.extend(
                            cell.get_resources_copy()
                                .iter()
                                .map(|resource| resource.as_str())
                                .collect::<Vec<&str>>(),
                        );
                        response.push(cell_response.join(" "));
                    }
                }
                vec![(player_id, ServerResponse::See(response))]
            }
            PlayerCommand::Inventory => {
                let player = self.players.get(&player_id).unwrap();
                let mut inventory = vec![format!(
                    "{} {}",
                    Resource::Nourriture,
                    player.remaining_life()
                )];
                inventory.extend(
                    player
                        .inventory()
                        .iter()
                        .enumerate()
                        .map(|(i, b)| format!("{} {}", Resource::try_from(i).unwrap(), b))
                        .collect::<Vec<String>>(),
                );
                vec![(player_id, ServerResponse::Inventory(inventory))]
            }
            PlayerCommand::Expel => {
                let (target_ids, direction) = {
                    let player = self.players.get(&player_id).unwrap();
                    let ids: Vec<u16> = self.map.field[player.position().y][player.position().x]
                        .players
                        .iter()
                        .filter_map(|&id| if id != player_id { Some(id) } else { None })
                        .collect();
                    (ids, player.position().direction)
                };
                let mut result: Vec<(u16, ServerResponse)> = target_ids
                    .iter()
                    .map(|&id| {
                        self.handle_move(id, &direction);
                        (id, ServerResponse::Movement(direction.opposite()))
                    })
                    .collect();
                result.push((player_id, ServerResponse::Ok));
                result
            }
            PlayerCommand::Broadcast { text } => {
                let sender_pos = self.players.get(&player_id).unwrap().position();
                self.players
                    .keys()
                    .map(|&id| {
                        let resp = if id == player_id {
                            ServerResponse::Ok
                        } else {
                            let receiver_pos = self.players.get(&id).unwrap().position();
                            ServerResponse::Message(
                                self.map.find_broadcast_source(sender_pos, receiver_pos),
                                text.clone(),
                            )
                        };
                        (id, resp)
                    })
                    .collect()
            }
            PlayerCommand::Incantation => {
                let player = self.players.get(&player_id).unwrap();
                let position = player.position();
                if *player.remaining_life() < PlayerCommand::INCANTATION_DURATION {
                    return vec![(player_id, ServerResponse::Ko)];
                }
                let same_lvl_players = self.map.field[position.y][position.x]
                    .players
                    .iter()
                    .filter_map(|&lvl| {
                        let other = self.players.get(&lvl).unwrap();
                        if *other.level() == *player.level()
                            && *other.remaining_life() >= PlayerCommand::INCANTATION_DURATION
                        {
                            Some(*other.id())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                if same_lvl_players.len() >= player.nxt_lvl_player_cnt_requirements()
                    && self.map.field[position.y][position.x]
                        .stones
                        .reduce_current_from(player.nxt_lvl_stone_requirements())
                {
                    same_lvl_players
                        .iter()
                        .map(|&id| {
                            self.incantation
                                .entry(self.frame + PlayerCommand::INCANTATION_DURATION)
                                .and_modify(|v| v.push(id))
                                .or_insert(vec![id]);
                            self.players.get_mut(&id).unwrap().start_incantation();
                            (id, ServerResponse::IncantationInProgress)
                        })
                        .collect()
                } else {
                    vec![(player_id, ServerResponse::Ko)]
                }
            }
            PlayerCommand::Fork => {
                // TODO: use MAX_PLAYERS to not abuse spamming
                let player = self.players.get(&player_id).unwrap();
                let egg = Egg {
                    team_name: player.team().clone(),
                    position: player.position().clone(),
                };
                self.eggs
                    .entry(self.frame + PlayerCommand::EGG_FETCH_TIME_DELAY)
                    .and_modify(|v| v.push(egg.clone()))
                    .or_insert(vec![egg]);
                self.map.field[player.position().y][player.position().x]
                    .eggs
                    .entry(player.team().clone())
                    .and_modify(|(unhatched, _)| *unhatched += 1)
                    .or_insert((1, 0));
                vec![(player_id, ServerResponse::Ok)]
            }
            PlayerCommand::ConnectNbr => {
                let team_name = self.players.get(&player_id).unwrap().team();
                vec![(
                    player_id,
                    ServerResponse::Value(
                        self.teams
                            .get(team_name)
                            .unwrap()
                            .remaining_members()
                            .to_string(),
                    ),
                )]
            }
        }
    }

    pub fn tick(&mut self, execution_results: &mut Vec<(u16, ServerResponse)>) {
        self.frame += 1;
        let current_frame = self.frame;
        let mut commands_to_process = Vec::new();

        let mut dead_players = HashSet::new();
        for (id, player) in &mut self.players {
            if *player.remaining_life() == 0 {
                log::info!(
                    "Player {} from {} died at ({}, {})",
                    player.id(),
                    player.team(),
                    player.position().x,
                    player.position().y
                );
                execution_results.push((*player.id(), ServerResponse::Mort));
                dead_players.insert(*player.id());
                continue;
            }

            player.decrement_life();

            if !player.commands().is_empty() && current_frame >= *player.next_frame() {
                let command = player.pop_command_from_queue().unwrap();
                player.set_next_frame(current_frame + command.delay());
                commands_to_process.push((*id, command));
            }
        }

        for player_id in dead_players {
            self.remove_player(player_id);
        }

        for (player_id, command) in commands_to_process {
            if *self
                .players
                .get(&player_id)
                .unwrap()
                .is_performing_incantation()
            {
                execution_results.push((player_id, ServerResponse::IncantationInProgress));
            } else {
                execution_results.extend(self.apply_cmd(player_id, &command));
            }
        }

        if let Some(eggs_to_hatch) = self.eggs.remove(&current_frame) {
            for egg in eggs_to_hatch {
                if let (Some((unhatched, hatched)), Some(team)) = (
                    self.map.field[egg.position.y][egg.position.x]
                        .eggs
                        .get_mut(&egg.team_name),
                    self.teams.get_mut(&egg.team_name),
                ) {
                    *unhatched -= 1;
                    *hatched += 1;
                    team.add_next_spawn_position(egg.position);
                    log::info!(
                        "Team {}: hatched egg at ({}, {})!",
                        egg.team_name,
                        egg.position.x,
                        egg.position.y
                    );
                }
            }
        }

        if let Some(players_to_stop_incantation) = self.incantation.remove(&current_frame) {
            for id in players_to_stop_incantation {
                let player = self.players.get_mut(&id).unwrap();
                execution_results.push((
                    id,
                    ServerResponse::CurrentLevel(player.stop_incantation().unwrap()), // TODO: handle level 8
                ));
            }
        }
    }

    pub fn add_player(&mut self, player_id: u16, team_name: String) -> Result<u16, ZappyError> {
        log::debug!("{player_id} wants to join {team_name}");
        let team = self.teams.get_mut(&team_name).unwrap();
        let pos = team.add_member(player_id)?;
        let player = Player::new(player_id, team_name.clone(), pos);
        self.map
            .add_player(*player.id(), player.team(), player.position());
        let log_successful_insert = format!(
            "The player with id: {} has successfully joined the \"{}\" team.",
            player.id(),
            player.team()
        );
        self.players.insert(player_id, player);
        log::info!("{log_successful_insert}");
        Ok(team.remaining_members())
    }

    pub fn remove_player(&mut self, player_id: u16) {
        if let Some(player) = self.players.remove(&player_id) {
            log::debug!("Client {player_id} has been removed from the server");
            self.map.remove_player(player.id(), player.position());
            self.teams
                .get_mut(player.team())
                .unwrap()
                .remove_member(player_id);
        }
    }

    pub fn take_command(
        &mut self,
        player_id: &u16,
        cmd: PlayerCommand,
    ) -> Result<Option<ServerResponse>, ZappyError> {
        if let Some(player) = self.players.get_mut(player_id) {
            Ok(if player.commands().len() >= MAX_COMMANDS {
                Some(ServerResponse::ActionQueueIsFull)
            } else {
                player.push_command_to_queue(cmd);
                None
            })
        } else {
            Err(Technical(IsNotConnectedToServer(*player_id)))
        }
    }

    pub fn map_width(&self) -> usize {
        *self.map.width()
    }

    pub fn map_height(&self) -> usize {
        *self.map.height()
    }
}
