#[macro_use]
extern crate lazy_static;
extern crate log;
extern crate simplelog;
pub mod scrape;
pub mod web_driver;
use chrono;
use colored::Colorize;
use serde::Serialize;
use std::error::Error;
use std::fs;
use std::path;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use thirtyfour::{
    prelude::{ElementWaitable, WebDriverError},
    By, DesiredCapabilities, WebDriver, WebElement,
};

use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode, WriteLogger};

#[derive(Debug, Serialize)]
pub struct Config {
    pub raw_data_dir: String,
}

lazy_static! {
    pub static ref CONFIG: Mutex<Config> = Mutex::new(Config {
        raw_data_dir: "data/raw/".to_string(),
    });
}

/// Init function check:
/// 1: if directory data exists, and create it if not
/// 2: if chromdriver is in the root directory, and panic if not
pub fn init() -> Result<(), Box<dyn Error>> {
    static HAS_RUN: Mutex<Option<bool>> = Mutex::new(Some(false));
    let mut has_run = HAS_RUN.lock().unwrap();
    {
        if *has_run.as_ref().unwrap() {
            println!("init() Already Run!!! This function shall only be called once");
            return Err("Bad!".into());
        }
    }
    *has_run = Some(true);

    let mut config = CONFIG.lock().unwrap();
    if !std::path::Path::new(&config.raw_data_dir).exists() {
        std::fs::create_dir_all(&config.raw_data_dir)?;
    }
    // TODO: automatic run chromedriver: new feature to be added
    // if !std::path::Path::new("chromedriver").exists() {
    //     panic!(
    //         "{}\n{}\n{}\n{}",
    //         "Chrome Driver does not exist!",
    //         "Download The Chrome Driver!".red().bold(),
    //         "This in unrecoverable error.",
    //         "Please Download the Chrome Driver with the same version as your browser. See readme.md"
    //     );
    // }

    // Initialise logging with simplelog

    // create data directory 
    fs::create_dir_all("data")?;

    // logging initialise
    let log_dir = String::from("log");
    fs::create_dir_all(&log_dir)?;
    let mut log_file_name: String =
        chrono::Local::now().format("%Y-%m-%d_%H:%M:%S_scraped_at").to_string() + ".txt";

    // if the file exists, wait a second to avoid overwriting
    if fs::metadata(&log_file_name).is_ok() {
        thread::sleep(Duration::from_millis(1020));
        log_file_name = chrono::Local::now().format("%Y-%m-%d_%H:%M:%S_scraped_at").to_string() + ".txt";
    }

    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Info,
            simplelog::Config::default(),
            fs::File::create(&format!("{}/{}", log_dir, log_file_name)).unwrap(),
        ),
        TermLogger::new(
            LevelFilter::Debug,
            simplelog::Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
    ])
    .unwrap();

    Ok(())
}
