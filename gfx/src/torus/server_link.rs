use crate::Message;
use bevy::prelude::*;
use shared::GFXData;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::sync::mpsc::UnboundedReceiver;

// TODO: don't clone and lock all this
#[derive(Resource)]
pub struct ServerLink {
    pub game_state: Arc<Mutex<Option<GFXData>>>,
    pub update: Arc<AtomicBool>,
}

impl ServerLink {
    pub fn new() -> Self {
        Self {
            game_state: Default::default(),
            update: Arc::new(false.into()),
        }
    }
}

pub fn network_setup(mut data_rx: UnboundedReceiver<Message>, server_link: &ServerLink) {
    let game_state = Arc::clone(&server_link.game_state);
    let update = Arc::clone(&server_link.update);

    tokio::spawn(async move {
        loop {
            let message = match data_rx.recv().await {
                Some(message) => message,
                None => {
                    *game_state.lock().unwrap() = None;
                    continue;
                }
            };
            match message {
                Message::Disconnect(error) => {
                    eprintln!("Failed to connect: {}, retrying in 1 second...", error);
                    *game_state.lock().unwrap() = None;
                }
                Message::State(new_state) => {
                    *game_state.lock().unwrap() = Some(new_state);
                }
            }
            update.store(true, Ordering::Relaxed);
        }
    });
}
