//! This module provide scrape function
//! which scrapes interns jobs posted on
//! "https://careers.airbnb.com/positions/?_departments=early-career-program-intern"
//! For all jobs, see  (they are not scraped)
//! "https://careers.airbnb.com/positions/"

use colored::Colorize;
use csv::Writer;
use log::*;
use std::error::{self, Error};
use std::fs::{self, OpenOptions};
use std::io::Write;
use thirtyfour::{
    prelude::{ElementQueryable, ElementWaitable, WebDriverError},
    By, WebDriver, WebElement,
};

// TODO: remove this dependency, use char
use chrono::NaiveDate;
use unicode_segmentation::UnicodeSegmentation;
use url::Url;

// total 8 fields
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

/// Holding errors info that shall be logged
/// all logging shall take place when the entries is recorded to csv
#[derive(Debug)]
pub struct ScrapeLogInfo {
    /// the row number in the data output csv file.
    pub row: usize,
    pub url: String,
}

// from and to are dates in the format "YYYY-MM-DD"
pub async fn scrape_from_to(
    driver: &WebDriver,
    from: &str,
    to: &str,
) -> Result<(), Box<dyn Error>> {
    let from_date = NaiveDate::parse_from_str(from, "%Y-%m-%d")?;
    let to_date = NaiveDate::parse_from_str(to, "%Y-%m-%d")?;

    if from_date > to_date {
        panic!("From date is later than to date. Internal Bug");
    }

    // iterate from from_date to to_date
    let mut current_date = from_date;

    while current_date <= to_date {
        scrape_date(driver, current_date).await?;
        current_date = current_date.succ_opt().unwrap();
    }

    Ok(())
}

pub async fn scrape_date(driver: &WebDriver, date: NaiveDate) -> Result<(), Box<dyn Error>> {
    // data directory is created in crate::init() defined in lib.rs
    let save_to = &format!("data/{}_shggzy", date.format("%Y-%m-%d"));

    let url = &format!("https://www.shggzy.com/search/queryContents_1.jhtml?title=&channelId=38&origin=&inDates=1&ext=&timeBegin={}&timeEnd={}%2B23%3A59%3A59&ext1=&ext2=&cExt=eyJhbGciOiJIUzI1NiJ9.eyJwYXRoIjoiL2p5eHh6YyIsInBhZ2VObyI6MSwiZXhwIjoxNzU2MTk3MTg4MDg3fQ.RpAdtIlYn7wkJDpA0rths1P5jlA0fbiaaWUJ6Kt8uz8", 
        date.format("%Y-%m-%d"),
        date.format("%Y-%m-%d"),
    );


    info!("{}, saving to {save_to}", "Scraping shggzy".yellow().bold(),);

    // FOR DEBUG:
    // let url = "http://localhost:3000";
    let url = "https://www.shggzy.com/jyxxzcgs/8466348?cExt=eyJhbGciOiJIUzI1NiJ9.eyJwYXRoIjoiL2p5eHh6YyIsInBhZ2VObyI6MSwiZXhwIjoxNzU2MTk3MTg4MDg3fQ.RpAdtIlYn7wkJDpA0rths1P5jlA0fbiaaWUJ6Kt8uz8&isIndex=";

    let url_tmp = Url::parse(url)?;
    driver.goto(url_tmp).await?;
    println!("{} at {}", "Scraping shggzy job".yellow().bold(), url);
    super::short_pause();

    let mut scraped_data: Vec<BidInfo> = Vec::new();
    let mut log_info: ScrapeLogInfo = ScrapeLogInfo {
        row: 1,
        url: url.into(),
    };

    // DEBUG:
    for i in scrape_bid_info(driver, &mut log_info).await? {
        write_log(&i, &log_info);
    }
    debug!("{:?}", scrape_bid_info(driver, &mut log_info).await?);
    panic!("{}", "Expected Panic!".red());
    // END DEBUG


    let mut i = 1;
    loop {
        // click_entry returns Ok(true) if the i th entry is clicked
        if !click_entry(driver, i).await? {
            i = 1;
            if !click_next_page(driver).await? {
                println!("No more pages to scrape; exiting...");
                break;
            }
            overcome_challenge(driver).await?;
            continue;
        } else {
            i += 1;
        }

        super::short_pause();

        super::swith_to_tab(driver, 1).await?;
        super::wait_until_loaded(driver).await?;

        log_info.url = driver.current_url().await?.to_string();
        for i in scrape_bid_info(driver, &mut log_info).await? {
            write_log(&i, &log_info);
            scraped_data.push(i);
            log_info.row += 1;
        }

        super::medium_pause();
        driver.close_window().await?;
        super::swith_to_tab(driver, 0).await?;
        super::short_pause();
    }

    save_bid_info(&scraped_data, save_to).await?;

    Ok(())
}

fn write_log(bid_info: &BidInfo, log_info: &ScrapeLogInfo) {
    let msg = "is empty (in write_log function). Likely wrong scraping logic or corrupted site.";
    if bid_info.project_id.is_empty() {
        warn!(
            "Project id {}      row: {}, url: {}",
            msg, log_info.row, log_info.url
        );
    }
    if bid_info.project_name.is_empty() {
        warn!(
            "project_name {}       row: {}, url: {}",
            msg, log_info.row, log_info.url
        );
    }
    if bid_info.recorded_date.is_empty() {
        warn!(
            "recorded date {}     row: {}, url: {}",
            msg, log_info.row, log_info.url
        );
    }
    if bid_info.company_name.is_empty() {
        warn!(
            "company name {}    row: {}, url: {}",
            msg, log_info.row, log_info.url
        );
    }
    if bid_info.company_address.is_empty() {
        warn!(
            "company address {}     row: {}, url: {}",
            msg, log_info.row, log_info.url
        );
    }
    if bid_info.price.is_empty() {
        warn!(
            "price {}     row: {}, url: {}",
            msg, log_info.row, log_info.url
        );
    }
    if bid_info.buyer.is_empty() {
        warn!(
            "buyer {}     row: {}, url: {}",
            msg, log_info.row, log_info.url
        );
    }
    if bid_info.publication_url.is_empty() {
        warn!(
            "publication url {}     row: {}, url: {}",
            msg, log_info.row, log_info.url
        );
    }
}

async fn scrape_bid_info(
    driver: &WebDriver,
    log_info: &mut ScrapeLogInfo,
) -> Result<Vec<BidInfo>, WebDriverError> {
    let mut ret: Vec<BidInfo> = vec![];
    match find_table(driver).await? {
        Some(t) => {
            ret.extend(handle_table(&t, log_info).await?);
        }
        // logging will take place when writing into csv
        None => ret.push(BidInfo::default()),
    }

    let project_id = get_project_id(driver).await?;
    let recorded_date = get_recorded_date(driver).await?;
    let buyer = get_buyer(driver).await?;
    let publication_url = get_publication_url(driver).await?;

    for i in ret.iter_mut() {
        i.recorded_date = recorded_date.clone();
        i.buyer = buyer.clone();
        i.publication_url = publication_url.clone();
        i.project_id = project_id.clone();
    }

    Ok(ret)
}

// save to json and csv
async fn save_bid_info(scraped_data: &[BidInfo], save_to: &str) -> Result<(), Box<dyn Error>> {
    // save the entries as json
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&format!("{}.json", save_to))?;
    let json_data = serde_json::to_string_pretty(&scraped_data)?;
    write!(file, "{}", json_data)?;

    info!(
        "{}",
        format!("Saved {} entries to {}.json", scraped_data.len(), save_to).green(),
    );

    let mut wtr = Writer::from_path(&format!("{}.csv", save_to))?;
    for i in scraped_data.iter() {
        // write each entry to csv
        wtr.serialize(i)?;
    }
    info!(
        "{}",
        format!("Saved {} entries to {}.csv", scraped_data.len(), save_to),
    );
    Ok(())
}

// todo: search by 下一页
async fn click_next_page(driver: &WebDriver) -> Result<bool, Box<dyn Error>> {
    if let Ok(next_page) = driver
        .find(By::XPath("//*[contains(text(),'下一页')]"))
        .await
    {
        next_page.wait_until().displayed().await?;
        // if the button is clickable, the attribute is layui-laypage-next
        // if not, it is layui-laypage-next layui-disabled
        // TODO: find a better way to check if the button is clickable
        let class_attribute = next_page.attr("class").await?;
        let attribute = class_attribute
            .as_deref()
            .expect("No class attribute found!");

        if attribute.contains("disabled") {
            debug!("Next page button is disabled; no more pages to scrape");
            return Ok(false);
        }

        next_page.click().await?;
        debug!("Clicked next page button");
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
async fn get_project_id(driver: &WebDriver) -> Result<String, WebDriverError> {
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
            return Ok(String::new());
        }
    }

    if project_id.is_empty() {
        return Ok(String::new());
    }

    Ok(project_id)
}

async fn get_recorded_date(driver: &WebDriver) -> Result<String, WebDriverError> {
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
    Ok(recorded_date)
}

async fn get_buyer(driver: &WebDriver) -> Result<String, WebDriverError> {
    let mut buyer_element = driver
        .find(By::XPath("//*[contains(text(),'采购人信息')]"))
        .await?;
    let mut buyer = String::new();
    // "名称：" has length 9
    while buyer.len() <= 9 {
        buyer = buyer_element
            .find(By::XPath("./following::*[1]"))
            .await?
            .text()
            .await?;

        buyer_element = buyer_element
            .find(By::XPath("./following::*[1]"))
            .await?;
    }

    buyer = buyer.replace("名称", "");
    buyer = buyer.replace("招标人", "");
    buyer = buyer.replace("：", "");
    buyer = buyer.trim().to_string();
    Ok(buyer)
}

async fn get_publication_url(driver: &WebDriver) -> Result<String, WebDriverError> {
    let mut publication_url = driver
        .find(By::XPath("//a[contains(text(),'http://')]"))
        .await?
        .text()
        .await?;

    publication_url = publication_url.trim().to_string();
    Ok(publication_url)
}

async fn find_table(driver: &WebDriver) -> Result<Option<WebElement>, WebDriverError> {
    let tables = driver.find_all(By::Tag("table")).await?;



    for (i, t) in tables.iter().enumerate() {
        let text = t.text().await?;
        if text.contains("地址") || text.contains("元") {
            return Ok(Some(t.clone()));
        }
    }

    Ok(None)
}

// This function returns a vector of Bidinfo, each holding the info scraped
// from each row of the table found by find_table
// Each entry of this vector will be added more information scraped from
// outside the table after this function returns to scrape_bid_info
// If there is non-panicing error, and no information is scraped
// return vector of length 0, or a vector of a single entry holding
// default value of Bidinfo (which is all empty string)
async fn handle_table(
    table: &WebElement,
    log_info: &mut ScrapeLogInfo,
) -> Result<Vec<BidInfo>, WebDriverError> {
    // tbody_entries are the rows, the first row may be the table head
    let mut tbody_entries: Vec<WebElement> = vec![];
    match table.find(By::Tag("tbody")).await {
        Ok(b) => {
            tbody_entries = b.find_all(By::XPath("./*")).await?;
        }
        Err(_) => {
            warn!("url: {}. no <tbody> found (in handle_table function). Likely wrong scraping logic or corrupted site. Scraping continue",log_info.url);
            return Ok(Vec::new());
        }
    }

    if tbody_entries.len() == 0 {
        warn!("url: {}. <tbody> has no entry (in handle_table function). Likely wrong scraping logic or corrupted site. Scraping continue",log_info.url);
        return Ok(vec![BidInfo::default()]);
    }

    // the table may have thead tag, or the head info is contained in
    // the first row of tbody (thead: table head, tbody: table body)
    let mut head_entries: Vec<String> = vec![];
    match table.find(By::Tag("thead")).await {
        Ok(t) => {
            let strings = t.text().await?;
            let strings: Vec<String> = strings.split(" ").map(|s| s.to_string()).collect();
            head_entries = strings;
        }
        Err(_) => {
            // since there is no thead, assume the first row of tbody is table head
            // if the table contains 0 or 1 lines, there is error
            let strings = tbody_entries[0].text().await?;

            // Now tbody contains only body, no head
            tbody_entries.remove(0);

            let strings: Vec<String> = strings.split(" ").map(|s| s.to_string()).collect();
            head_entries = strings;
        }
    }

    if tbody_entries.len() == 0 {
        warn!("url: {}. <tbody> has no entry (in handle_table function). Likely wrong scraping logic or corrupted site. Scraping continue",log_info.url);
    }

    // upon this point, we have found the table body and head

    // this the table stored vec
    // formated_table[0][1] is the 0' row, 1st col
    let mut formatted_table: Vec<Vec<String>> = vec![];
    for r in tbody_entries.iter() {
        let cols_entry: Vec<String> = r.text().await?.split(" ").map(|s| s.to_string()).collect();

        formatted_table.push(cols_entry);
    }

    // The table head usually is like
    // 序号 标项名称 中标供应商名称
    // 中标供应商地址 评审报价 评审总得分 中标（成交金额） 备注
    // but the order may change, and the name may change
    // we need to find the index for each required entry
    let mut project_name_idx: Option<usize> = None;
    let mut company_name_idx: Option<usize> = None;
    let mut company_addr_idx: Option<usize> = None;
    let mut price_idx: Option<usize> = None;

    for (i, s) in head_entries.iter().enumerate() {
        if s.contains("金额") && price_idx.is_none() {
            price_idx = Some(i);
        } else if s.contains("标项") && project_name_idx.is_none() {
            project_name_idx = Some(i)
        } else if s.contains("地址") && company_addr_idx.is_none() {
            company_addr_idx = Some(i);
        } else if s.contains("供应商") && s.contains("名称") && company_name_idx.is_none() {
            company_name_idx = Some(i);
        }
    }

    let mut ret: Vec<BidInfo> = Vec::new();
    for i in formatted_table.iter() {
        let mut tmp = BidInfo {
            ..Default::default()
        };
        if let Some(idx) = project_name_idx {
            tmp.project_name = i[idx].trim().to_string();
        }
        if let Some(idx) = company_name_idx {
            tmp.company_name = i[idx].trim().to_string();
        }
        if let Some(idx) = company_addr_idx {
            tmp.company_address = i[idx].trim().to_string();
        }
        if let Some(idx) = price_idx {
            tmp.price = i[idx].trim().to_string();
        }
        ret.push(tmp);
    }

    Ok(ret)
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
        debug!("click_entry: nothing to click; continuing...");
        return Ok(false);
    }
    Ok(true)
}

// sometime there is a robot challenge after clicking next page
// in such case, the driver is pasued, and require user intervention
// the user shall solve the challenge, and the scraping program will continue
// to work
// The challenge is a drag and fill in the shape puzzle
async fn overcome_challenge(driver: &WebDriver) -> Result<(), Box<dyn Error>> {
    loop {
        debug!("{}", "Waiting for the page to load...");
        match driver
            // this is the 下一页 button
            // If the buttons shows, the page is loaded
            .find(By::XPath("//*[contains(text(),'下一页')]"))
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
        super::long_pause();
    }
    Ok(())
}
