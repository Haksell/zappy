use super::ServerData;
use bevy::prelude::*;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use tokio::sync::mpsc::Receiver;

// TODO: don't clone and lock all this
#[derive(Resource)]
pub struct ServerLink {
    pub data_rx: Arc<Mutex<Receiver<ServerData>>>,
    pub game_state: Arc<Mutex<ServerData>>,
    pub update: Arc<AtomicBool>,
}

impl ServerLink {
    pub fn new(data_rx: Receiver<ServerData>) -> Self {
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
                    tokio::select! {
                        Some(new_data) = data_rx.recv() => {
                            *game_state.lock().unwrap() = new_data;
                            update.store(true, Ordering::Relaxed);
                        }
                        // Helps not crashing when closing bevy. TODO: find a better way?
                        _ = tokio::time::sleep(Duration::from_millis(50)) => {} // TODO: check best sleep
                    }
                }
            });
    });
}
