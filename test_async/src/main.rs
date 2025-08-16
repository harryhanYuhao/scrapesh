#[macro_use]
extern crate log;

use colored::Colorize;
use hypochlorite::CONFIG;
use hypochlorite::{
    scrape::{
        self,
        shggzy::{self, BidInfo},
    },
    web_driver,
};
use serde::Serialize;
use thirtyfour::prelude::ElementQueryable;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use thirtyfour::{
    prelude::{ElementWaitable, WebDriverError},
    By, DesiredCapabilities, Key, WebDriver, WebElement,
};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    hypochlorite::init()?;
    // killguard has custom drop traits that kill the subprocess
    // It has to be declared here: no easy alternative
    let _kill_guard = web_driver::KillChildGuard;
    let driver = web_driver::init_driver(web_driver::DriverType::Default).await?;

    scrape::medium_pause();

    scrape(&driver, "shggzy.json").await?;
    // scrape::shggzy::read_bid_info_json_save_csv("shggzy_bid_info_copy.json", "july-30.csv").await?;

    driver.quit().await?;
    Ok(())
}

pub async fn scrape(driver: &WebDriver, save_to: &str) -> Result<(), Box<dyn Error>> {
    println!("{}, saving to {save_to}", "Scraping shggzy".yellow().bold(),);

    // let url = "https://www.shggzy.com/search/queryContents_1.jhtml?title=&channelId=38&origin=&inDates=1&ext=&timeBegin=2025-07-31&timeEnd=2025-8-6%2B23%3A59%3A59&ext1=&ext2=&cExt=eyJhbGciOiJIUzI1NiJ9.eyJwYXRoIjoiL2p5eHh6YyIsInBhZ2VObyI6MSwiZXhwIjoxNzU2MTk3MTg4MDg3fQ.RpAdtIlYn7wkJDpA0rths1P5jlA0fbiaaWUJ6Kt8uz8";

    let url = "http://localhost:3000";

    // let url = "https://www.shggzy.com/search/queryContents_11.jhtml?title=&channelId=38&origin=&inDates=30&ext=&timeBegin=2025-07-07&timeEnd=2025-8-7%3A59%3A59&ext1=&ext2=&cExt=eyJhbGciOiJIUzI1NiJ9.eyJwYXRoIjoiL2p5eHh6YyIsInBhZ2VObyI6MSwiZXhwIjoxNzU2MTk3MTg4MDg3fQ.RpAdtIlYn7wkJDpA0rths1P5jlA0fbiaaWUJ6Kt8uz8";

    let url_tmp = Url::parse(url)?;
    driver.goto(url_tmp).await?;
    debug!("{} at {}", "Scraping shggzy job", url);
    scrape::short_pause();

    loop {
        println!("{}", "Waiting for the page to load...".yellow().bold(),);
        match driver
            .find(By::XPath("/html/body/div[6]/div[3]/div[1]/div[2]/h4"))
            .await
        {
            Ok(_) => {
                debug!("{}", "Page loaded successfully");
                break;
            }
            Err(_) => {
                debug!("Page not loaded yet, retrying...");
            }
        }
        scrape::medium_pause();
    }

    scrape::short_pause();

    // TEST:
    // let body = driver.find(By::Tag("body")).await?;
    // body.send_keys(Key::Control + "s").await?;
    // scrape::long_pause();
    scrape::scroll_to_bottom(&driver).await?;

    let mut ret: Vec<BidInfo> = Vec::new();
    scrape::short_pause();

    // scrape::swith_to_tab(driver, 1).await?;
    // scrape::wait_until_loaded(driver).await?;

    ret.push(scrape_bid_info(driver).await?);

    scrape::medium_pause();
    // driver.close_window().await?;
    // scrape::swith_to_tab(driver, 0).await?;
    scrape::short_pause();
    // save the entries as json

    println!(
        "{}",
        format!("Saved {} entries to {}", ret.len(), save_to).green(),
    );

    Ok(())
}

async fn scrape_bid_info(driver: &WebDriver) -> Result<BidInfo, WebDriverError> {

    let mut table = driver
        .query(By::Tag("tbody"))
        .first()
        .await?;
    println!("Table found: {:?}", table);

    return Ok(BidInfo::default());

    let mut project_id = driver
        .find(By::XPath("/html/body/div[6]/div[3]/div[1]/div[2]/h4"))
        .await?
        .text()
        .await?;
    project_id = project_id.trim().to_string();
    project_id.retain(|c| c.is_digit(10) || c == '-');

    let project_name = driver
        .find(By::XPath(
            "/html/body/div[6]/div[3]/div[4]/div[2]/ul[2]/li[2]",
        ))
        .await?
        .text()
        .await?
        .trim()
        .to_string();

    // This field looks like
    // 发布时间：2025-07-31     信息来源：上海市财政局云平台    浏览次数：130
    // we only date the first 16 characters
    let recorded_date = driver
        .find(By::XPath("/html/body/div[6]/div[3]/p"))
        .await?
        .text()
        .await?
        .trim()
        .to_string();

    let characters: Vec<char> = recorded_date.chars().collect();
    let mut recorded_date: String = characters[..16].iter().collect();

    recorded_date = recorded_date.trim().to_string();
    recorded_date.retain(|c| c.is_digit(10) || c == '-');

    let mut price = driver
        .find(By::XPath(
            "/html/body/div[6]/div[3]/div[4]/div[2]/ul[7]/li[2]/div/div[1]/table/tbody/tr/td[3]",
        ))
        .await?
        .text()
        .await?;
    price = price.trim().to_string();
    price.retain(|c| c.is_digit(10) || c == '.');

    let mut company_name = driver
        .find(By::XPath(
            "/html/body/div[6]/div[3]/div[4]/div[2]/ul[7]/li[2]/div/div[1]/table/tbody/tr/td[4]",
        ))
        .await?
        .text()
        .await?;
    company_name = company_name.trim().to_string();

    let company_address = driver
        .find(By::XPath(
            "/html/body/div[6]/div[3]/div[4]/div[2]/ul[7]/li[2]/div/div[1]/table/tbody/tr[1]/td[5]",
        ))
        .await?
        .text()
        .await?
        .trim()
        .to_string();

    let buyer_element = driver
        .find(By::XPath("//*[contains(text(),'采购人信息')]"))
        .await?;
    let buyer = buyer_element
        .find(By::XPath("./following::*[1]"))
        .await?
        .text()
        .await?
        .replace("名 称：", "");

    let mut publication_url = driver
        .find(By::XPath("//*[contains(text(),'http:')]"))
        .await?
        .attr("href")
        .await?
        .unwrap_or_default();
    publication_url = publication_url.trim().to_string();

    let ret = BidInfo {
        project_id,
        project_name,
        recorded_date,
        price,
        company_name,
        company_address,
        buyer,
        publication_url,
        ..Default::default() // default defined in main
    };
    println!("{:?}", ret);
    Ok(ret)
}
