pub mod error;

use error::Error;

use ftail::{ansi_escape::TextStyling, Config, Ftail};
use log::{Level, LevelFilter, Log};

pub fn init_logger(dir: &str, log_level: LevelFilter) -> Result<(), Error> {
    match Ftail::new()
        .custom(
            |config: ftail::Config| Box::new(CustomLogger { config }) as Box<dyn Log + Send + Sync>,
            log_level,
        )
        .daily_file(dir, log_level)
        .datetime_format("%Y-%m-%d %H:%M:%S")
        .init()
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(e.to_string().as_str())),
    }
}

// the custom logger implementation
struct CustomLogger {
    config: Config,
}

impl Log for CustomLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        if self.config.level_filter == LevelFilter::Off {
            return true;
        }

        metadata.level() <= self.config.level_filter
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let time = chrono::Local::now()
            .format(&self.config.datetime_format)
            .to_string();

        match record.level() {
            Level::Error => println!(
                "{}{}{} {} {}",
                "[".blue(),
                "E".red().bold(),
                "]".blue(),
                time.bright_black(),
                record.args().red()
            ),
            Level::Warn => println!(
                "{}{}{} {} {}",
                "[".blue(),
                "W".yellow().bold(),
                "]".blue(),
                time.bright_black(),
                record.args().yellow()
            ),
            Level::Info => println!(
                "{}{}{} {} {}",
                "[".blue(),
                "i".green().bold(),
                "]".blue(),
                time.bright_black(),
                record.args()
            ),
            Level::Debug => println!(
                "{}{}{} {} {}",
                "[".blue(),
                "D".blue().bold(),
                "]".blue(),
                time.bright_black(),
                record.args().blue()
            ),
            Level::Trace => println!(
                "{}{}{} {} {}",
                "[".blue(),
                "T".magenta().bold(),
                "]".blue(),
                time.bright_black(),
                record.args().magenta()
            ),
        }
    }

    fn flush(&self) {}
}
