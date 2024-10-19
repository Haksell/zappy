use crate::args::ServerArgs;
use shared::player::Player;
use shared::{Command, Map, ServerCommandToClient, ServerResponse, ZappyError, MAX_COMMANDS};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct Server {
    pub(crate) width: usize,
    pub(crate) height: usize,
    max_clients: u16,
    pub(crate) tud: u16,
    teams: HashMap<String, Vec<Arc<Player>>>,
    clients: HashMap<u16, Arc<Player>>,
    pub(crate) map: Map,
    pub(crate) frame: u64,
}

impl Server {
    pub async fn from(args: &ServerArgs) -> Result<Self, Box<dyn Error>> {
        let teams = args
            .names
            .iter()
            .map(|k| (k.clone(), Vec::with_capacity(args.clients as usize)))
            .collect();
        Ok(Self {
            width: args.width,
            height: args.height,
            max_clients: args.clients,
            tud: args.tud,
            teams,
            clients: HashMap::new(),
            map: Map::new(args.width, args.height),
            frame: 0,
        })
    }

    //TODO: it is launched in the loop that borrows self
    // so it can't be self, investigate is it the best place for this logic?
    fn execute(map: &Map, player: &mut Player, command: &Command) -> Option<ServerResponse> {
        log::debug!("Executing command: {:?} for {:?}", command, player);
        Some(ServerResponse::Mort)
    }

    //TODO: C like approach to send execution results to the player but I can't see now
    // a better way to quit this server lock and don't reallocate memory for responses each teak
    pub fn tick(&mut self, execution_results: &mut Vec<(u16, ServerResponse)>) {
        //self.map.next_position();
        for (_, player) in &mut self.clients {
            // TODO: handle 0-time differently
            if !player.commands.is_empty() && self.frame >= player.next_frame {
                let player_mut = Arc::make_mut(player);
                let command = player_mut.commands.pop_front().unwrap();
                if let Some(resp) = Server::execute(&self.map, player_mut, &command) {
                    execution_results.push((player_mut.id(), resp));
                }
                player_mut.next_frame = self.frame + command.delay();
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
        log::debug!("{player_id} wants to join {}", team);
        let team_name_trimmed = team.trim().to_string();
        let remaining_clients = self.remaining_clients(&team_name_trimmed)?;
        if remaining_clients > 0 {
            let (x, y) = self.map.random_position();
            let player = Arc::new(Player::new(communication_channel, player_id, team_name_trimmed.clone(), x, y));
            self.map.add_player(Arc::clone(&player));
            self.teams.get_mut(&team_name_trimmed).unwrap().push(Arc::clone(&player));
            if let Some(_) = self.clients.insert(player_id, player) {
                //TODO: is it possible? need to handle?
                log::warn!("Duplicate connection attempted from {player_id}.");
            }
            Ok((remaining_clients - 1) as usize)
        } else {
            Err(ZappyError::MaxPlayersReached)
        }
    }

    pub fn remove_player(&mut self, player_id: &u16) {
        if let Some(player) = self.clients.remove(player_id) {
            log::debug!("Client {player_id} has been removed from the server");
            self.teams.get_mut(&player.team).unwrap().retain(|p| p.id() != *player_id);
            //player.disconnect().await?;
        }
    }

    pub fn remaining_clients(&self, team_name: &str) -> Result<u16, ZappyError> {
        if let Some(players_in_team) = self.teams.get(team_name) {
            Ok(self.max_clients - players_in_team.len() as u16)
        } else {
            Err(ZappyError::TeamDoesntExist)
        }
    }

    pub fn take_command(&mut self, player_id: &u16, cmd: Command) -> Result<(), ZappyError> {
        if let Some(player) = self.clients.get_mut(player_id) {
            if player.commands.len() >= MAX_COMMANDS {
                // TODO: send message
                log::debug!("Player {player_id:?} tried to push {cmd:?} in to a full queue.");
                return Err(ZappyError::Waring(ServerResponse::ActionQueueIsFull));
            } else {
                Arc::make_mut(player).commands.push_back(cmd);
            }
            Ok(())
        } else {
            Err(ZappyError::IsNotConnectedToServer)
        }
    }
}
