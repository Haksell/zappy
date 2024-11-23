use crate::args::ServerArgs;
use derive_getters::Getters;
use shared::player::Direction;
use shared::LogicalError::{MaxPlayersReached, TeamDoesntExist};
use shared::TechnicalError::IsNotConnectedToServer;
use shared::ZappyError::{Logical, Technical};
use shared::{
    commands::PlayerCommand,
    map::Map,
    player::{Player, Side},
    resource::Resource,
    ServerResponse, ZappyError, MAX_COMMANDS,
};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

#[derive(Debug, Getters)]
pub struct GameEngine {
    max_clients: u16,
    teams: HashMap<String, HashSet<u16>>,
    players: HashMap<u16, Player>,
    map: Map,
    frame: u64,
}

impl GameEngine {
    pub async fn from(args: &ServerArgs) -> Result<Self, Box<dyn Error>> {
        let teams = args
            .names
            .iter()
            .map(|k| (k.clone(), HashSet::with_capacity(args.clients as usize)))
            .collect();
        Ok(Self {
            max_clients: args.clients,
            teams,
            players: HashMap::new(),
            map: Map::new(args.width, args.height),
            frame: 0,
        })
    }

    fn handle_move(&mut self, player_id: u16, direction: &Direction) {
        let player = self.players.get_mut(&player_id).unwrap();
        let current_x = player.position().x;
        let current_y = player.position().y;

        let (dx, dy) = direction.dx_dy();
        player.set_x(((current_x + self.map.width()) as isize + dx) as usize % self.map.width());
        player.set_y(((current_y + self.map.height()) as isize + dy) as usize % self.map.height());

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
                let result = Resource::try_from(resource_name.as_str())
                    .map(|resource| {
                        let player = self.players.get_mut(&player_id).unwrap();
                        let cell = &mut self.map.field[player.position().y][player.position().x];
                        if cell.resources[resource as usize] >= 1 {
                            cell.resources[resource as usize] -= 1;
                            player.add_to_inventory(resource);
                            (player_id, ServerResponse::Ok)
                        } else {
                            (player_id, ServerResponse::Ko)
                        }
                    })
                    .unwrap_or((player_id, ServerResponse::Ko));
                vec![result]
            }
            PlayerCommand::Put { resource_name } => {
                let result = Resource::try_from(resource_name.as_str())
                    .map(|resource| {
                        let player = self.players.get_mut(&player_id).unwrap();
                        let cell = &mut self.map.field[player.position().y][player.position().x];
                        if player.remove_from_inventory(resource) {
                            cell.resources[resource as usize] += 1;
                            (player_id, ServerResponse::Ok)
                        } else {
                            (player_id, ServerResponse::Ko)
                        }
                    })
                    .unwrap_or((player_id, ServerResponse::Ko));
                vec![result]
            }
            PlayerCommand::See => todo!(),
            PlayerCommand::Inventory => {
                let player = self.players.get_mut(&player_id).unwrap();
                let inventory = player
                    .inventory()
                    .iter()
                    .enumerate()
                    .map(|(i, b)| format!("{} {}", Resource::try_from(i as u8).unwrap(), b))
                    .collect::<Vec<String>>();
                vec![(player_id, ServerResponse::Inventory(inventory))]
            }
            PlayerCommand::Expel => {
                let (target_ids, direction) = {
                    let player = self.players.get_mut(&player_id).unwrap();
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
                        (id, ServerResponse::Movement(direction.opposite_side()))
                    })
                    .collect();
                result.push((player_id, ServerResponse::Ok));
                result
            }
            PlayerCommand::Broadcast { .. } => todo!(),
            PlayerCommand::Incantation => todo!(),
            PlayerCommand::Fork => todo!(),
            PlayerCommand::ConnectNbr => todo!(),
        }
    }

    pub fn tick(&mut self, execution_results: &mut Vec<(u16, ServerResponse)>) {
        self.frame += 1;
        let current_frame = self.frame;

        let mut commands_to_process = Vec::new();
        for (id, player) in &mut self.players {
            if !player.commands().is_empty() && current_frame >= *player.next_frame() {
                let command = player.pop_command_from_queue().unwrap();
                player.set_next_frame(current_frame + command.delay());
                commands_to_process.push((*id, command));
            }
        }

        for (player_id, command) in commands_to_process {
            execution_results.extend(self.apply_cmd(player_id, &command));
        }
    }

    pub fn add_player(&mut self, player_id: u16, team: String) -> Result<usize, ZappyError> {
        log::debug!("{player_id} wants to join {team}");
        let remaining_clients = self.remaining_clients(&team)?;
        if remaining_clients > 0 {
            let player = Player::new(player_id, team.clone(), self.map.random_position());
            self.map.add_player(*player.id(), player.position());
            self.teams.get_mut(&team).unwrap().insert(*player.id());
            let log_successful_insert = format!(
                "The player with id: {} has successfully joined the \"{}\" team.",
                player.id(),
                player.team()
            );
            self.players.insert(player_id, player);
            log::info!("{log_successful_insert}");
            Ok((remaining_clients - 1) as usize)
        } else {
            Err(Logical(MaxPlayersReached(player_id, remaining_clients)))
        }
    }

    pub fn remove_player(&mut self, player_id: &u16) {
        if let Some(player) = self.players.remove(player_id) {
            log::debug!("Client {player_id} has been removed from the server");
            self.map.remove_player(player.id(), player.position());
            self.teams.get_mut(player.team()).unwrap().remove(player_id);
        }
    }

    pub fn remaining_clients(&self, team_name: &str) -> Result<u16, ZappyError> {
        if let Some(players_in_team) = self.teams.get(team_name) {
            Ok(self.max_clients - players_in_team.len() as u16)
        } else {
            Err(Logical(TeamDoesntExist(team_name.to_string())))
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
