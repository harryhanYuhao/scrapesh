//! This module provide scrape function
//! which scrapes interns jobs posted on
//! "https://careers.airbnb.com/positions/?_departments=early-career-program-intern"
//! For all jobs, see  (they are not scraped)
//! "https://careers.airbnb.com/positions/"

use colored::Colorize;
use csv::Writer;
use log::*;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use thirtyfour::{
    prelude::{ElementQueryable, ElementWaitable, WebDriverError},
    By, WebDriver, WebElement,
};

// TODO: remove this dependency, use char
use unicode_segmentation::UnicodeSegmentation;
use url::Url;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct BidInfo {
    // 项目编号
    pub project_id: String,

    // 标项名称 or 
    pub project_name: String,

    // 存证日期
    // todo: change to Date type
    pub recorded_date: String,

    // 中标供应商名称
    pub company_name: String,

    // 中标供应商地址
    pub company_address: String,

    // 中标（成交金额）
    // unit: CNY
    pub price: String,

    // 采购人信息
    pub buyer: String,

    pub publication_url: String,
}

//  A true value means the field is obtained
//  A field with true value denotes the required info is scraped
//  The default for bool is false
//  Default can be used as
//  let mut bid_info_scraped = Default::default();
#[derive(Debug, Default)]
struct ScrapeTracker {
    pub project_id: bool,
    pub project_name: bool,

    // 存证日期
    // todo: change to Date type
    pub recorded_date: bool,
    pub company_name: bool,
    pub company_address: bool,
    // unit: CNY
    pub price: bool,

    // 采购人信息
    pub buyer: bool,

    pub publication_url: bool,
}

/// Holding errors info that shall be logged
#[derive(Debug)]
pub struct ScrapeLogInfo {
    /// the row number in the data output csv file.
    pub row: usize,
    pub url: String,
}

pub async fn scrape(driver: &WebDriver, save_to: &str) -> Result<(), Box<dyn Error>> {
    info!("{}, saving to {save_to}", "Scraping shggzy".yellow().bold(),);

    let url = "https://www.shggzy.com/search/queryContents_1.jhtml?title=&channelId=38&origin=&inDates=1&ext=&timeBegin=2025-07-31&timeEnd=2025-8-6%2B23%3A59%3A59&ext1=&ext2=&cExt=eyJhbGciOiJIUzI1NiJ9.eyJwYXRoIjoiL2p5eHh6YyIsInBhZ2VObyI6MSwiZXhwIjoxNzU2MTk3MTg4MDg3fQ.RpAdtIlYn7wkJDpA0rths1P5jlA0fbiaaWUJ6Kt8uz8";
    // let url = "https://www.shggzy.com/search/queryContents_1.jhtml?title=&channelId=38&origin=&inDates=1&ext=&timeBegin=2025-07-30&timeEnd=2025-7-30%2B23%3A59%3A59&ext1=&ext2=&cExt=eyJhbGciOiJIUzI1NiJ9.eyJwYXRoIjoiL2p5eHh6YyIsInBhZ2VObyI6MSwiZXhwIjoxNzU2MTk3MTg4MDg3fQ.RpAdtIlYn7wkJDpA0rths1P5jlA0fbiaaWUJ6Kt8uz8";

    let url = "http://localhost:3000";

    let url_tmp = Url::parse(url)?;
    driver.goto(url_tmp).await?;
    println!("{} at {}", "Scraping shggzy job".yellow().bold(), url);
    super::short_pause();

    let mut scraped_data: Vec<BidInfo> = Vec::new();
    let mut log_info: ScrapeLogInfo = ScrapeLogInfo {
        row: 0,
        url: url.into(),
    };
    let mut scrape_tracker = ScrapeTracker::default();

    // DEBUG:
    scrape_bid_info(driver, &mut scrape_tracker, &mut log_info).await?;

    let mut i = 1;
    loop {
        // DEBUG:
        break;

        if !click_entry(driver, i).await? {
            i = 1;
            if !click_next_page(driver).await? {
                println!("No more pages to scrape; exiting...");
                break;
            }
            continue;
        } else {
            i += 1;
        }

        // WARNING: FOR DEBUG
        // if i == 3 {
        //     break;
        // }

        super::short_pause();

        super::swith_to_tab(driver, 1).await?;
        super::wait_until_loaded(driver).await?;

        scraped_data.push(scrape_bid_info(driver, &mut scrape_tracker, &mut log_info).await?);

        super::medium_pause();
        driver.close_window().await?;
        super::swith_to_tab(driver, 0).await?;
        super::short_pause();
    }

    save_bid_info(&scraped_data, save_to).await?;

    Ok(())
}

async fn scrape_bid_info(
    driver: &WebDriver,
    scrape_tracker: &mut ScrapeTracker,
    log_info: &mut ScrapeLogInfo,
) -> Result<BidInfo, WebDriverError> {
    let mut table = driver.query(By::Tag("tbody")).first().await?;
    println!("Table found: {:?}", table);

    return Ok(BidInfo::default());

    let mut project_id: String;
    if let Some(project_id) = get_project_id(driver).await? {
        scrape_tracker.project_id = true;
    } else {
        scrape_tracker.project_id = false;
    }

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

// save to json and csv
async fn save_bid_info(scraped_data: &[BidInfo], save_to: &str) -> Result<(), Box<dyn Error>> {
    // save the entries as json
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&format!("{},json", save_to))?;
    let json_data = serde_json::to_string_pretty(&scraped_data)?;
    write!(file, "{}", json_data)?;

    info!(
        "{}",
        format!("Saved {} entries to {}", scraped_data.len(), save_to).green(),
    );

    let mut wtr = Writer::from_path(&format!("{}.csv", save_to))?;
    for i in scraped_data.iter() {
        // write each entry to csv
        wtr.serialize(i)?;
    }
    info!(
        "{}",
        format!("Saved {} entries to {}", scraped_data.len(), save_to),
    );
    Ok(())
}

// todo: search by 下一页
async fn click_next_page(driver: &WebDriver) -> Result<bool, Box<dyn Error>> {
    if let Ok(next_page) = driver
        .find(By::XPath(
            "/html/body/div[5]/div/div/div/div[2]/div[3]/div[1]/div/div/div/a[7]",
        ))
        .await
    {
        next_page.wait_until().displayed().await?;
        // if the button is clickable, the attribute is layui-laypage-next
        // if not, it is layui-laypage-next layui-disabled
        let class_attribute = next_page.attr("class").await?;
        let attribute = class_attribute
            .as_deref()
            .expect("No class attribute found!");

        if attribute.contains("disabled") {
            debug!("Next page button is disabled; no more pages to scrape");
            return Ok(false);
        }

        next_page.click().await?;
        return Ok(true);
    }
    println!("No next page button found");
    return Ok(false);
}

fn first_half_unicode(s: &str) -> String {
    let graphemes: Vec<&str> = s.graphemes(true).collect();
    graphemes[..graphemes.len() / 2].concat()
}

// Returns Ok(None) is no project ID is found, this is likely due to uncorrect scraping logic
// Return Err() if there is other program error
async fn get_project_id(driver: &WebDriver) -> Result<Option<String>, WebDriverError> {
    let mut project_id: String;
    match driver
        .find(By::XPath("/html/body/div[6]/div[3]/div[1]/div[2]/h4"))
        .await
    {
        Ok(element) => {
            project_id = element.text().await?;
            project_id = project_id.trim().to_string();
            project_id.retain(|c| c.is_digit(10) || c == '-');
        }
        Err(_) => {
            return Ok(None);
        }
    }

    if project_id.is_empty() {
        return Ok(None);
    }

    Ok(Some(project_id))
}

// for serializing to csv

// return true if clicked, false if not
async fn click_entry(driver: &WebDriver, number: usize) -> Result<bool, Box<dyn Error>> {
    if let Ok(result_entry) = driver
        .find(By::XPath(&format!(
            "/html/body/div[5]/div/div/div/div[2]/div[3]/div[1]/ul/li[{number}]",
        )))
        .await
    {
        result_entry.wait_until().clickable().await?;
        result_entry.click().await?;
    } else {
        println!("click_entry: nothing to click; continuing...");
        return Ok(false);
    }
    return Ok(true);
}

/// this site does not seem to have a popup menu
// async fn click_popup(driver: &WebDriver) -> Result<(), Box<dyn Error>> {
//     if let Ok(popup_menu_ok_button) = driver
//         .find(By::XPath("/html/body/dialog[1]/div[2]/button[2]"))
//         .await
//     {
//         popup_menu_ok_button.wait_until().clickable().await?;
//         popup_menu_ok_button.click().await?;
//         return Ok(());
//     }
//     println!("No popup menu found; continuing...");
//     return Ok(());
// }

async fn get_all_entry(driver: &WebDriver) -> Result<Vec<WebElement>, WebDriverError> {
    driver
        .find_all(By::XPath("/html/body/div/div[2]/div[2]/div"))
        .await
}
