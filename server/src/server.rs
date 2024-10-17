use crate::args::ServerArgs;
use crate::map::Play;
use crate::player::Player;
use crate::{ServerCommandToClient, ZappyError};
use shared::{Command, Map, MAX_COMMANDS};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;

pub struct Server {
    port: u16,
    pub(crate) width: usize,
    pub(crate) height: usize,
    max_clients: u16,
    pub(crate) tud: u16,
    team_names: Vec<String>,
    clients: HashMap<SocketAddr, Player>,
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

    pub fn tick(&mut self) {
        //self.map.next_position();
        for (_, player) in &mut self.clients {
            // TODO: handle 0-time differently
            if !player.commands.is_empty() && self.frame >= player.next_frame {
                let command = player.commands.pop_front().unwrap();
                player.execute(&command);
                player.next_frame = self.frame + command.delay();
            }
        }
        self.frame += 1;
    }

    //TODO: maybe bed idea to disconnect client here, because this method is called during the mutex lock
    pub async fn add_player(
        &mut self,
        addr: SocketAddr,
        communication_channel: Sender<ServerCommandToClient>,
        team: String,
    ) -> Result<(), ZappyError> {
        log::debug!("{addr} wants to join {}", team);
        if !self.team_names.contains(&team.trim().into()) {
            Err(ZappyError::TeamDoesntExist)
        } else if self.remaining_clients() == 0 {
            Err(ZappyError::MaxPlayersReached)
        } else {
            let id = self.get_available_ids();
            if let Some(dup) = self
                .clients
                .insert(addr, Player::new(communication_channel, id, team))
            {
                log::error!("Duplicate connection attempted from {addr}. Disconnecting both...");
                dup.disconnect().await?;
                self.remove_player(&addr).await?;
            }
            Ok(())
        }
    }

    pub async fn remove_player(&mut self, addr: &SocketAddr) -> Result<(), ZappyError> {
        if let Some(player) = self.clients.remove(addr) {
            log::debug!("Client removed {addr}, sending shutdown");
            player.disconnect().await?;
            Ok(())
        } else {
            log::debug!("{addr} isn't connected");
            Err(ZappyError::TryToDisconnectNotConnected)
        }
    }

    pub fn remaining_clients(&self) -> u16 {
        self.max_clients - self.clients.len() as u16
    }

    pub fn take_command(&mut self, addr: &SocketAddr, cmd: Command) {
        let client = self.clients.get_mut(addr).unwrap();
        if client.commands.len() >= MAX_COMMANDS {
            // TODO: send message
            log::debug!("Client {addr:?} tried to push {cmd:?} in to a full queue.");
        } else {
            client.commands.push_back(cmd);
        }
    }

    fn get_available_ids(&mut self) -> usize {
        let id = self.client_max_id;
        self.client_max_id = self.client_max_id + 1;
        id
    }
}
