//! This module provide scrape function
//! which scrapes interns jobs posted on
//! "https://careers.airbnb.com/positions/?_departments=early-career-program-intern"
//! For all jobs, see  (they are not scraped)
//! "https://careers.airbnb.com/positions/"

use log::*;
use colored::Colorize;
use csv::Writer;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use thirtyfour::{
    prelude::{ElementWaitable, WebDriverError},
    By, WebDriver, WebElement,
};
// TODO: remove this dependency, use char
use unicode_segmentation::UnicodeSegmentation;
use url::Url;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct BidInfo {
    // 项目编号
    pub project_id: String,
    pub project_name: String,

    // 存证日期
    // todo: change to Date type
    pub recorded_date: String,
    pub company_name: String,
    pub company_address: String,
    // unit: CNY
    pub price: String,

    // 采购人信息
    pub buyer: String,

    pub publication_url: String,
}

pub async fn read_bid_info_json_save_csv(
    read_from: &str,
    save_to: &str,
) -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string(read_from)?;
    let entries: Vec<BidInfo> = serde_json::from_str(&data)?;
    println!("Read {} entries from shggzy_bid_info.json", entries.len());

    let mut wtr = Writer::from_path(save_to)?;
    for i in entries.iter() {
        // write each entry to csv
        wtr.serialize(i)?;
    }
    println!(
        "{}",
        format!("Saved {} entries to {}", entries.len(), save_to).green(),
    );
    Ok(())
}

pub async fn scrape(driver: &WebDriver, save_to: &str) -> Result<(), Box<dyn Error>> {
    println!("{}, saving to {save_to}", "Scraping shggzy".yellow().bold(),);

    let url = "https://www.shggzy.com/search/queryContents_1.jhtml?title=&channelId=38&origin=&inDates=1&ext=&timeBegin=2025-07-31&timeEnd=2025-8-6%2B23%3A59%3A59&ext1=&ext2=&cExt=eyJhbGciOiJIUzI1NiJ9.eyJwYXRoIjoiL2p5eHh6YyIsInBhZ2VObyI6MSwiZXhwIjoxNzU2MTk3MTg4MDg3fQ.RpAdtIlYn7wkJDpA0rths1P5jlA0fbiaaWUJ6Kt8uz8";
    // let url = "https://www.shggzy.com/search/queryContents_1.jhtml?title=&channelId=38&origin=&inDates=1&ext=&timeBegin=2025-07-30&timeEnd=2025-7-30%2B23%3A59%3A59&ext1=&ext2=&cExt=eyJhbGciOiJIUzI1NiJ9.eyJwYXRoIjoiL2p5eHh6YyIsInBhZ2VObyI6MSwiZXhwIjoxNzU2MTk3MTg4MDg3fQ.RpAdtIlYn7wkJDpA0rths1P5jlA0fbiaaWUJ6Kt8uz8";

    let url_tmp = Url::parse(url)?;
    driver.goto(url_tmp).await?;
    println!("{} at {}", "Scraping shggzy job".yellow().bold(), url);
    super::short_pause();

    let mut ret: Vec<BidInfo> = Vec::new();

    let mut i = 1;
    loop {
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

        ret.push(scrape_bid_info(driver).await?);

        super::medium_pause();
        driver.close_window().await?;
        super::swith_to_tab(driver, 0).await?;
        super::short_pause();
    }
    // save the entries as json
    let mut file = OpenOptions::new().write(true).create(true).open(save_to)?;
    let json_data = serde_json::to_string_pretty(&ret)?;
    write!(file, "{}", json_data)?;

    println!(
        "{}",
        format!("Saved {} entries to {}", ret.len(), save_to).green(),
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

// for serializing to csv
async fn scrape_bid_info(driver: &WebDriver) -> Result<BidInfo, WebDriverError> {
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
    let mut recorded_date : String =characters[..16].iter().collect();

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
