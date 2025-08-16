#[macro_use]
extern crate log;

mod scrape;
mod web_driver;
use colored::Colorize;
use hypochlorite::CONFIG;
use serde::Serialize;
use std::error::Error;
use std::sync::Mutex;
use thirtyfour::{
    prelude::{ElementWaitable, WebDriverError},
    By, DesiredCapabilities, WebDriver, WebElement,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    hypochlorite::init()?;

    // killguard has custom drop traits that kill the subprocess
    // It has to be declared here: no easy alternative
    let _kill_guard = web_driver::KillChildGuard;
    let driver = web_driver::init_driver(web_driver::DriverType::Default).await?;

    scrape::short_pause();
    scrape::shggzy::scrape(&driver, "shggzy").await?;
    info!("aaa");

    driver.quit().await?;
    Ok(())
}
