#[macro_use]
extern crate log;

mod scrape;
mod web_driver;

use chrono::NaiveDate;
use hypochlorite::CONFIG;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    hypochlorite::init()?;

    // killguard has custom drop traits that kill the subprocess
    // It has to be declared here: no easy alternative
    let _kill_guard = web_driver::KillChildGuard;
    let driver = web_driver::init_driver(web_driver::DriverType::Default).await?;

    scrape::short_pause();
    scrape::shggzy::scrape_from_to(&driver, "2025-8-13", "2025-8-17").await?;
    info!("Scraping Finished Successfully!");

    driver.quit().await?;
    Ok(())
}
