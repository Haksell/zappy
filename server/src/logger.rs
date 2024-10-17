use chrono::Local;
use colored::Colorize;
use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;

pub fn init_logger() {
    Builder::new()
        .filter(None, LevelFilter::Debug)
        .format(|buf, record| {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S.%3f");
            let level = match record.level() {
                Level::Error => "ERROR".red().bold(),
                Level::Warn => "WARN".yellow().bold(),
                Level::Info => "INFO".green().bold(),
                Level::Debug => "DEBUG".blue().bold(),
                Level::Trace => "TRACE".magenta().bold(),
            };
            writeln!(
                buf,
                "{} [{}]: {}",
                timestamp,
                level,
                record.args().to_string().trim_end()
            )
        })
        .init();
    log::debug!("Starting the server");
}
