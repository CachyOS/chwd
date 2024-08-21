use std::env;

use colored::Colorize;
use log::{Level, Metadata, Record};

struct SimpleLogger;

static LOGGER: SimpleLogger = SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_str = match record.level() {
                Level::Error => "Error:".red(),
                Level::Warn => "Warning:".yellow(),
                Level::Info => ">".red(),
                Level::Debug => "Debug:".white(),
                Level::Trace => "Trace:".black(),
            };
            if record.level() == Level::Error {
                eprintln!("{level_str} {}", record.args());
            } else {
                println!("{level_str} {}", record.args());
            }
        }
    }

    fn flush(&self) {
        // use std::io::Write;
        // io::stdout().flush().unwrap();
    }
}

pub fn init_logger() -> Result<(), log::SetLoggerError> {
    // set log level
    let max_log_level = if let Ok(env_log) = env::var("RUST_LOG") {
        let env_log = env_log.to_lowercase();
        match env_log.as_str() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            _ => log::LevelFilter::Info,
        }
    } else {
        log::LevelFilter::Info
    };

    log::set_logger(&LOGGER).map(|()| log::set_max_level(max_log_level))
}
