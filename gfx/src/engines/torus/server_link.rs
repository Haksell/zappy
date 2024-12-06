use crate::Message;

use super::ServerData;
use bevy::prelude::*;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};
use tokio::sync::mpsc::UnboundedReceiver;

// TODO: don't clone and lock all this
#[derive(Resource)]
pub struct ServerLink {
    pub data_rx: Arc<Mutex<UnboundedReceiver<Message>>>,
    pub game_state: Arc<Mutex<Option<ServerData>>>,
    pub update: Arc<AtomicBool>,
}

impl ServerLink {
    pub fn new(data_rx: UnboundedReceiver<Message>) -> Self {
        Self {
            data_rx: Arc::new(Mutex::new(data_rx)),
            game_state: Default::default(),
            update: Arc::new(false.into()),
        }
    }
}

pub fn network_setup(server_link: ResMut<ServerLink>) {
    let data_rx = Arc::clone(&server_link.data_rx);
    let game_state = Arc::clone(&server_link.game_state);
    let update = Arc::clone(&server_link.update);

    thread::spawn(move || {
        let mut data_rx = data_rx.lock().unwrap();

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                loop {
                    let message = match data_rx.recv().await {
                        Some(message) => message,
                        None => {
                            eprintln!("None in recv ????");
                            continue;
                        }
                    };
                    match message {
                        Message::Disconnect => {
                            *game_state.lock().unwrap() = None;
                            update.store(true, Ordering::Relaxed);
                        }
                        Message::Data(new_data) => {
                            *game_state.lock().unwrap() = Some(new_data);
                            update.store(true, Ordering::Relaxed);
                        }
                        _ => {}
                    }
                }
            });
    });
}
