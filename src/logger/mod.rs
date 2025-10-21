pub mod error;

use colored::Colorize;
use error::Error;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::io::Write;

/// Displays the ASCII art copyright notice.
pub fn display_copyright() {
    let ascii_art = r#"
                                                      88
                                                      ""    ,d
                                                            88
    ,adPPYba,  8b       d8  8b,dPPYba,   8b,     ,d8  88  MM88MMM
    I8[    ""  `8b     d8'  88P'   `"8a   `Y8, ,8P'   88    88
     `"Y8ba,    `8b   d8'   88       88     )888(     88    88
    aa    ]8I    `8b,d8'    88       88   ,d8" "8b,   88    88,
    `"YbbdP"'      Y88'     88       88  8P'     `Y8  88    "Y888
                   d8'
                  d8'       "#;
    println!(
        "{}{} {} {}\n",
        ascii_art.yellow().bold(),
        "(c)".blue().bold(),
        "2021-2025".bright_black(),
        "the synxit developers".yellow().bold()
    );
}

/// Initializes the logger with the specified directory and log level.
pub fn init_logger(dir: &str, log_level: LevelFilter) -> Result<(), Error> {
    log::set_boxed_logger(Box::new(Logger {
        log_dir: dir.to_string(),
        level_filter: log_level,
    }))
    .map_err(|e| Error::new(e.to_string().as_str()))?;
    log::set_max_level(log_level);
    Ok(())
}

/// Custom logger implementation for handling log messages.
struct Logger {
    log_dir: String,
    level_filter: LevelFilter,
}

impl Log for Logger {
    /// Determines if a log message should be logged based on its metadata.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    /// Logs a record, handling single-line and multi-line messages appropriately.
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let message = record.args().to_string();

        if message.contains('\n') {
            self.log_multiline(record.level(), &message, &time);
        } else {
            self.log_single_line(record.level(), &message, &time);
        }
    }

    fn flush(&self) {}
}

impl Logger {
    /// Logs a single-line message to both terminal and file.
    fn log_single_line(&self, level: Level, message: &str, time: &str) {
        log_to_terminal(level, message, time);
        self.write_to_file(level, message, time);
    }

    /// Logs a multi-line message with decorative borders to both terminal and file.
    fn log_multiline(&self, level: Level, message: &str, time: &str) {
        self.log_single_line(level, "┏━━", time);
        for line in message.lines() {
            self.log_single_line(level, &format!("┃ {}", line), time);
        }
        self.log_single_line(level, "┗━━", time);
    }

    /// Writes a log message to the appropriate log file.
    fn write_to_file(&self, level: Level, message: &str, time: &str) {
        let plain_message = format!("[{}] {} {}", level, time, message);
        // file name = YYYY-MM-DD.log
        let file_name = chrono::Local::now().format("%Y-%m-%d.log").to_string();
        let file_path = format!("{}/{}", &self.log_dir, file_name);
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
        {
            if let Err(e) = writeln!(file, "{}", plain_message) {
                eprintln!("Failed to write to log file: {}", e);
            }
        }
    }
}

/// Logs a message to the terminal with appropriate color coding.
fn log_to_terminal(level: Level, message: &str, time: &str) {
    let (level_char, level_color) = match level {
        Level::Error => ("E", "red"),
        Level::Warn => ("W", "yellow"),
        Level::Info => ("i", "green"),
        Level::Debug => ("D", "blue"),
        Level::Trace => ("T", "magenta"),
    };

    println!(
        "{}{}{} {} {}",
        "[".blue(),
        level_char.color(level_color).bold(),
        "]".blue(),
        time.bright_black(),
        message.color(level_color)
    );
}
