use crate::ServerCommandToClient;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;

pub struct Server {
    pub max_clients: u32,
    pub clients: HashMap<SocketAddr, Sender<ServerCommandToClient>>,
}

impl Server {
    pub fn new(max_clients: u32) -> Self {
        Self {
            max_clients,
            clients: HashMap::new(),
        }
    }

    //TODO: implement teams
    pub fn add_client(
        &mut self,
        team_name: String,
        addr: SocketAddr,
        receiver: Sender<ServerCommandToClient>,
    ) -> bool {
        log::debug!("{addr} wants to join {team_name}");
        if self.remaining_clients() == 0 {
            false
        } else {
            //TODO: handle not inserted
            self.clients.insert(addr, receiver);
            true
        }
    }

    pub fn remove_client(&mut self, addr: &SocketAddr) -> bool {
        if let Some(sender) = self.clients.remove(addr) {
            log::debug!("Client removed {addr}, sending shutdown");
            let _ = sender.send(ServerCommandToClient::Shutdown);
            true
        } else {
            log::debug!("{addr} isn't connected");
            false
        }
    }

    pub fn remaining_clients(&self) -> u32 {
        self.max_clients - self.clients.len() as u32
    }
}
