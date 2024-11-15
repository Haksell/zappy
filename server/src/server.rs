use crate::args::ServerArgs;
use shared::{
    command::Command,
    map::Map,
    player::{Player, Side},
    resource::Resource,
    ServerCommandToClient, ServerResponse, ZappyError, MAX_COMMANDS,
};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};
use tokio::sync::mpsc::Sender;

pub struct Server {
    pub(crate) width: usize,
    pub(crate) height: usize,
    max_clients: u16,
    pub(crate) _tud: u16,
    pub teams: HashMap<String, HashSet<u16>>,
    pub(crate) players: HashMap<u16, Player>,
    pub(crate) map: Map,
    pub(crate) frame: u64,
}

impl Server {
    pub async fn from(args: &ServerArgs) -> Result<Self, Box<dyn Error>> {
        let teams = args
            .names
            .iter()
            .map(|k| (k.clone(), HashSet::with_capacity(args.clients as usize)))
            .collect();
        Ok(Self {
            width: args.width,
            height: args.height,
            max_clients: args.clients,
            _tud: args.tud,
            teams,
            players: HashMap::new(),
            map: Map::new(args.width, args.height),
            frame: 0,
        })
    }

    fn handle_avance(&mut self, player_id: u16) {
        let player = self.players.get_mut(&player_id).unwrap();
        let current_x = player.position().x;
        let current_y = player.position().y;

        let (dx, dy) = player.position().direction.dx_dy();
        player.set_x(((current_x + self.map.width) as isize + dx) as usize % self.map.width);
        player.set_y(((current_y + self.height) as isize + dy) as usize % self.map.height);

        self.map.field[current_y][current_x]
            .players
            .remove(player.id());
        self.map.field[player.position().y][player.position().x]
            .players
            .insert(*player.id());
    }

    fn apply_cmd(&mut self, player_id: u16, command: &Command) -> Option<ServerResponse> {
        let player = self.players.get_mut(&player_id).unwrap();
        log::debug!("Executing command: {:?} for {:?}", command, player);
        match command {
            Command::Gauche => {
                player.turn(Side::Left);
                Some(ServerResponse::Ok)
            }
            Command::Droite => {
                player.turn(Side::Right);
                Some(ServerResponse::Ok)
            }
            Command::Avance => {
                self.handle_avance(player_id);
                Some(ServerResponse::Ok)
            }
            Command::Prend { resource_name } => {
                if let Ok(resource) = Resource::try_from(resource_name.as_str()) {
                    let cell = &mut self.map.field[player.position().y][player.position().x];
                    if cell.resources[resource as usize] >= 1 {
                        cell.resources[resource as usize] -= 1;
                        player.add_to_inventory(resource);
                        return Some(ServerResponse::Ok);
                    }
                }
                Some(ServerResponse::Ko)
            }
            Command::Pose { resource_name } => {
                if let Ok(resource) = Resource::try_from(resource_name.as_str()) {
                    let cell = &mut self.map.field[player.position().y][player.position().x];
                    if player.remove_from_inventory(resource) {
                        cell.resources[resource as usize] += 1;
                        return Some(ServerResponse::Ok);
                    }
                }
                Some(ServerResponse::Ko)
            }
            Command::Voir => todo!(),
            Command::Inventaire => {
                let inventory = player
                    .inventory()
                    .iter()
                    .enumerate()
                    .map(|(i, b)| format!("{} {}", Resource::try_from(i as u8).unwrap(), b))
                    .collect::<Vec<String>>();
                Some(ServerResponse::Inventory(inventory))
            }
            Command::Expulse => todo!(),
            Command::Broadcast { .. } => todo!(),
            Command::Incantation => todo!(),
            Command::Fork => todo!(),
            Command::ConnectNbr => todo!(),
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
            if let Some(resp) = self.apply_cmd(player_id, &command) {
                execution_results.push((player_id, resp));
            }
        }
    }

    pub fn add_player(
        &mut self,
        player_id: u16,
        communication_channel: Sender<ServerCommandToClient>,
        team: String,
    ) -> Result<usize, ZappyError> {
        log::debug!("{player_id} wants to join {team}");
        let team_name_trimmed = team.trim().to_string();
        let remaining_clients = self.remaining_clients(&team_name_trimmed)?;
        if remaining_clients > 0 {
            let player = Player::new(
                communication_channel,
                player_id,
                team_name_trimmed.clone(),
                self.map.random_position(),
            );
            self.map.add_player(*player.id(), player.position());
            self.teams
                .get_mut(&team_name_trimmed)
                .unwrap()
                .insert(*player.id());
            let log_successful_insert = format!(
                "The player with id: {} has successfully joined the \"{}\" team.",
                player.id(),
                player.team()
            );
            self.players.insert(player_id, player);
            log::info!("{log_successful_insert}");
            Ok((remaining_clients - 1) as usize)
        } else {
            Err(ZappyError::MaxPlayersReached)
        }
    }

    pub fn remove_player(&mut self, player_id: &u16) {
        if let Some(player) = self.players.remove(player_id) {
            log::debug!("Client {player_id} has been removed from the server");
            self.map.remove_player(player.id(), player.position());
            self.teams.get_mut(player.team()).unwrap().remove(player_id);
            //player.disconnect().await?;
        }
    }

    pub fn remaining_clients(&self, team_name: &str) -> Result<u16, ZappyError> {
        if let Some(players_in_team) = self.teams.get(team_name) {
            Ok(self.max_clients - players_in_team.len() as u16)
        } else {
            Err(ZappyError::TeamDoesntExist(team_name.to_string()))
        }
    }

    pub fn take_command(
        &mut self,
        player_id: &u16,
        cmd: Command,
    ) -> Result<Option<ServerResponse>, ZappyError> {
        if let Some(player) = self.players.get_mut(player_id) {
            Ok(if player.commands().len() >= MAX_COMMANDS {
                // TODO: send message
                log::debug!("Player {player_id:?} tried to push {cmd:?} in to a full queue.");
                Some(ServerResponse::ActionQueueIsFull)
            } else {
                player.push_command_to_queue(cmd);
                None
            })
        } else {
            Err(ZappyError::IsNotConnectedToServer)
        }
    }
}
