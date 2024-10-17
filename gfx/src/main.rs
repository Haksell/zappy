use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use serde_json::from_str;
use shared::{Map, GFX_PORT};
use std::io::{stdout, Write};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use unicode_width::UnicodeWidthChar;

pub trait PrettyPrint {
    fn pretty_print(&self);
}

impl PrettyPrint for Map {
    fn pretty_print(&self) {
        execute!(stdout(), Clear(ClearType::All)).unwrap();
        for row in &self.map {
            for &ch in row {
                let width = UnicodeWidthChar::width(ch).unwrap_or(1);
                // TODO: clean print!("{:1$}", ch, 3 - width);
                if width == 2 {
                    print!("{:2}", ch);
                } else {
                    print!("{:3}", ch);
                }
            }
            println!();
        }
        stdout().flush().unwrap();
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect(format!("127.0.0.1:{GFX_PORT}")).await?;
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        match from_str::<Map>(&line) {
            Ok(data) => {
                data.pretty_print();
            }
            Err(e) => {
                eprintln!("Failed to deserialize JSON: {}", e);
            }
        }
    }

    Ok(())
}
