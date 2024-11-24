use crate::line_reader::LineReader;
use rustyline::error::ReadlineError;

mod line_reader;

const PROMPT: &'static str = ">> ";

fn handle_login(line_reader: &mut LineReader) -> Result<(), ReadlineError> {
    let username = line_reader.readline_prompt("Username: ")?;
    let password = line_reader.read_secret("Password: ")?;

    let answer = line_reader.readline_prompt("Do you want to show data? (y/n): ")?;
    match answer.as_str() {
        "y" => println!("Username: {}, password: {}", username, password),
        &_ => println!("Bye"),
    }

    loop {
        println!("{}", line_reader.readline()?)
    }
}

fn main() -> Result<(), ReadlineError> {
    let mut line_reader = LineReader::new(PROMPT.to_string())?;

    handle_login(&mut line_reader)?;

    Ok(())
}
