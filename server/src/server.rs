use crate::args::ServerArgs;
use shared::{
    command::Command, player::Player, Map, ServerCommandToClient, ServerResponse, ZappyError,
    MAX_COMMANDS,
};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tokio::sync::mpsc::Sender;

pub struct Server {
    pub(crate) width: usize,
    pub(crate) height: usize,
    max_clients: u16,
    pub(crate) tud: u16,
    teams: HashMap<String, HashSet<u16>>,
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
            tud: args.tud,
            teams,
            players: HashMap::new(),
            map: Map::new(args.width, args.height),
            frame: 0,
        })
    }

    //TODO: C like approach to send execution results to the player but I can't see now
    // a better way to quit this server lock and don't reallocate memory for responses each teak
    pub fn tick(&mut self, execution_results: &mut Vec<(u16, ServerResponse)>) {
        let map = &mut self.map;
        for (_, player) in &mut self.players {
            // TODO: handle 0-time differently
            if !player.commands().is_empty() && self.frame >= *player.next_frame() {
                let command = player.pop_command_from_queue().unwrap();
                if let Some(resp) = map.apply_cmd(player, &command) {
                    execution_results.push((*player.id(), resp));
                }
                player.set_next_frame(self.frame + command.delay());
            }
        }
        self.frame += 1;
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
