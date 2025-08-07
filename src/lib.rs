#[macro_use]
extern crate lazy_static;

mod scrape;
mod web_driver;
use colored::Colorize;
use serde::Serialize;
use std::error::Error;
use std::sync::Mutex;
use thirtyfour::{
    prelude::{ElementWaitable, WebDriverError},
    By, DesiredCapabilities, WebDriver, WebElement,
};

#[derive(Debug, Serialize, Default)]
pub struct JobEntry {
    // company_name: String,
    pub company_name: String,
    pub job_title: String,
    pub apply_link: String,
    pub job_type_time: String,
    pub is_rolling: bool,
    pub location: String,
    pub ddl: String,
    pub start_time: String,
    pub duration: String,
    pub salary: String,
    pub description: String,
    pub keyworkds: String,
}

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
    Ok(())
}
