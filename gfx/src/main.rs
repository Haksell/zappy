use ratatui::layout::Margin;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    widgets::Block,
};
use ratatui::{
    layout::{Constraint, Layout},
    Frame,
};
use serde_json::from_str;
use shared::Map;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::Duration;

fn draw(frame: &mut Frame, data: &Option<Map>) {
    if let Some(data) = data {
        let area = frame.area().inner(Margin {
            vertical: 1,
            horizontal: 1,
        });

        let rows =
            Layout::vertical(vec![Constraint::Ratio(1, data.width as u32); data.width]).split(area);

        let cols = rows.iter().flat_map(|row| {
            Layout::horizontal(vec![Constraint::Ratio(1, data.height as u32); data.height])
                .split(*row)
                .to_vec()
        });

        for col in cols {
            frame.render_widget(Block::bordered().title("COORD"), col);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();

    let (event_tx, mut event_rx) = mpsc::channel(100);
    tokio::spawn(async move {
        loop {
            let poll = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(500)))
                .await
                .unwrap();

            if let Ok(true) = poll {
                let evt = tokio::task::spawn_blocking(|| event::read()).await.unwrap();
                if let Ok(Event::Key(key)) = evt {
                    if event_tx.send(key).await.is_err() {
                        break;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });

    let (tx, mut rx) = mpsc::channel(100);
    tokio::spawn(async move {
        let stream = TcpStream::connect("127.0.0.1:4343").await.unwrap();
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            match from_str::<Map>(&line) {
                Ok(data) => {
                    if tx.send(data).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to deserialize JSON: {}", e);
                }
            }
        }
    });

    let mut data: Option<Map> = None;
    loop {
        terminal.draw(|frame| {
            draw(frame, &data);
        })?;
        tokio::select! {
            Some(key) = event_rx.recv() => {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
            Some(new_data) = rx.recv() => {
                data = Some(new_data);
            }

            //_ = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    }
    ratatui::restore();
    Ok(())
}
