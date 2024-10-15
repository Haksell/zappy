use crate::client::Client;
use std::collections::HashMap;

pub struct Server {
    pub max_clients: u32,
    pub client_max_id: u64,
    pub clients: HashMap<u64, Client>,
}

impl Server {
    pub fn new(max_clients: u32) -> Self {
        Self {
            max_clients,
            client_max_id: 0,
            clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, client: Client) -> bool {
        if self.remaining_clients() == 0 {
            false
        } else {
            //self.clients.push(format!("{:?}", client.as_fd()));
            true
        }
    }

    pub fn remaining_clients(&self) -> u32 {
        self.max_clients - self.clients.len() as u32
    }
}
