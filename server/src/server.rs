use crate::args::ServerArgs;
use shared::player::{MessageToPlayer, Player};
use shared::{Command, Map, ServerCommandToClient, ZappyError, MAX_COMMANDS};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;

pub struct Server {
    port: u16,
    pub(crate) width: usize,
    pub(crate) height: usize,
    max_clients: u16,
    pub(crate) tud: u16,
    team_names: Vec<String>,
    clients: HashMap<SocketAddr, Arc<Player>>,
    client_max_id: usize,
    pub(crate) map: Map,
    pub(crate) frame: u64,
}

impl Server {
    pub async fn from(args: &ServerArgs) -> Result<(Self, TcpListener), Box<dyn Error>> {
        let addr = format!("127.0.0.1:{}", args.port);
        let listener = TcpListener::bind(&addr).await?;
        log::debug!("Listening on: {}", addr);
        Ok((
            Self {
                port: args.port,
                width: args.width,
                height: args.height,
                max_clients: args.clients,
                tud: args.tud,
                team_names: args.names.clone(),
                clients: HashMap::new(),
                client_max_id: 0,
                map: Map::new(args.width, args.height),
                frame: 0,
            },
            listener,
        ))
    }
    
    //TODO: it is launched in the loop that borrows self
    // so it can't be self, investigate is it the best place for this logic?
    fn execute(map: &Map, player: &mut Player, command: &Command) {
        log::debug!("Executing command: {:?} for {:?}", command, player);
    }

    pub fn tick(&mut self) {
        //self.map.next_position();
        for (_, player) in &mut self.clients {
            // TODO: handle 0-time differently
            if !player.commands.is_empty() && self.frame >= player.next_frame {
                let player_mut = Arc::make_mut(player);
                let command = player_mut.commands.pop_front().unwrap();
                player_mut.next_frame = self.frame + command.delay();
                Server::execute(&self.map, player_mut, &command);
            }
        }
        self.frame += 1;
    }

    pub fn add_player(
        &mut self,
        addr: SocketAddr,
        communication_channel: Sender<ServerCommandToClient>,
        team: String,
    ) -> Result<(), ZappyError> {
        log::debug!("{addr} wants to join {}", team);
        if !self.team_names.contains(&team.trim().into()) {
            Err(ZappyError::TeamDoesntExist)
        } else if self.remaining_clients() == 0 {
            // TODO: for each team
            Err(ZappyError::MaxPlayersReached)
        } else {
            let id = self.get_available_ids();
            let (x, y) = self.map.random_position();
            let player = Arc::new(Player::new(communication_channel, id, team, x, y));
            self.map.add_player(Arc::clone(&player));
            if let Some(_) = self.clients.insert(addr, player) {
                //TODO: is it possible? need to handle?
                log::warn!("Duplicate connection attempted from {addr}.");
            }
            Ok(())
        }
    }

    pub fn remove_player(&mut self, addr: &SocketAddr) -> Result<(), ZappyError> {
        if let Some(_) = self.clients.remove(addr) {
            log::debug!("Client removed {addr} from the server");
            //player.disconnect().await?;
            Ok(())
        } else {
            log::error!("{addr} isn't connected");
            Err(ZappyError::TryToDisconnectNotConnected)
        }
    }

    pub fn remaining_clients(&self) -> u16 {
        self.max_clients - self.clients.len() as u16
    }

    pub fn take_command(&mut self, addr: &SocketAddr, cmd: Command) -> Result<(), ZappyError> {
        if let Some(player) = self.clients.get_mut(addr) {
            if player.commands.len() >= MAX_COMMANDS {
                // TODO: send message
                log::debug!("Player {addr:?} tried to push {cmd:?} in to a full queue.");
                return Err(ZappyError::Waring(MessageToPlayer::ActionQueueIsFull))
            } else {
                Arc::make_mut(player).commands.push_back(cmd);
            }
            Ok(())
        } else {
            Err(ZappyError::IsNotConnectedToServer)
        }
    }

    fn get_available_ids(&mut self) -> usize {
        let id = self.client_max_id;
        self.client_max_id = self.client_max_id + 1;
        id
    }
}
