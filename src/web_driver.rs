use std::error::Error;
use std::process::{Child, Command};
use log::*;
use thirtyfour::{
    prelude::{ElementWaitable, WebDriverError},
    By, DesiredCapabilities, WebDriver, WebElement,
};

use colored::Colorize;
use std::sync::Mutex;

static CHILD: Mutex<Option<Child>> = Mutex::new(None);

pub struct KillChildGuard;

// For automatically killing the child process
// need to be initialised in main function.
impl Drop for KillChildGuard {
    fn drop(&mut self) {
        let child = CHILD.lock().unwrap().take();
        if let Some(mut child) = child {
            println!("Killing ChromeDriver ... ");
            child.kill().expect("failed to kill");
        }
    }
}

fn run_chrome_driver() {
    if std::path::Path::new("chromedriver").exists() {
        debug!("Chromdriver found! Running it now  ...");
        let child = Command::new("./chromedriver")
            .arg("--port=9515")
            .spawn()
            .expect("Failed To Run Chromedriver");
        *CHILD.lock().unwrap() = Some(child);
    } else {
        panic!(
            "{}\n{}\n{}",
            "Chrome Driver does not exist!",
            "Download The Chrome Driver!".red().bold(),
            "Please Download the Chrome Driver with the same version as your browser. See readme.md"
            );
    }
}

pub enum DriverType {
    Custom,
    Default,
}

/// Initialize and run the driver
/// This function shall only be called once
pub async fn init_driver(
    use_custom_driver: DriverType,
) -> Result<WebDriver, Box<dyn Error>> {
    static HAS_RUN: Mutex<Option<bool>> = Mutex::new(Some(false));
    let mut has_run = HAS_RUN.lock().unwrap();
    {
        if *has_run.as_ref().unwrap() {
            panic!("initialize_driver() Already Run! This function shall only be called once. This is likely internal bug.");
        }
    }
    *has_run = Some(true);

    match use_custom_driver {
        DriverType::Custom => {
            println!("Using Custom Chrome Driver");
        }
        DriverType::Default => {
            println!("Using Default Chrome Driver: Booting up Driver");
            run_chrome_driver();
            // Wait for the driver to boot up
            crate::scrape::medium_pause();
        }
    }

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps)
        .await
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                "Failed To Connects to Chrome Driver at port 9515"
                    .bold()
                    .red()
            )
        });
    // WARNING: does not seem to work in hyprland
    // driver.maximize_window().await?;
    Ok(driver)
}
