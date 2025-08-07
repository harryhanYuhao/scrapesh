pub mod shggzy;
use rand::Rng;
use serde::Serialize;
use std::error::Error;
use std::thread;
use std::time::Duration;
use thirtyfour::{
    prelude::{ElementWaitable, WebDriverError},
    By, DesiredCapabilities, WebDriver, WebElement,
};
use url::Url;

pub async fn raw_scrape(url: &str) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::builder().build()?;
    let mut headers = reqwest::header::HeaderMap::new();
    // some dummy headers to behave like real human
    headers.insert("Cookie", "E00AD1D372569A8FB07933566EDDBB4C~000000000000000000000000000000~YAAQXqcQAs/CCZqOAQAAVQE0mhd8Ihh1M43UDUC9y53EG2NViL1EM1OTuMv6YoVTrZrd0zXBl5znA02S0iFeU3+EZS3FaFs/A26n09got4DW5ZpJwSmU06DpRyMlWaTNe8bXOI9sPKD5wU9i1nXXilJJTf5h/49sgALv202IzTSURWJVDGxodDtcpxhKlXQHi42Cm9BwpEWDy1FSnhujk8bR7lZDKAmPdA87mY1X3pD64J3Uti7/hptFm+2Ui6sXeTfV19QIY+azGEDGIUPSyq/8GbJmOK/qBbfrDdDQEvuWBw7Trjnf/ZhFYW6G5dmBrWrzdleyvZmDCjWCMF3dkh1BE2nK/DZN4ZKGejKp4lUvHDOVJzP8Zri10jtn6t2wq/kI4dNloRznjeU=".parse()?);
    let url = Url::parse(url)?;

    let request = client.request(reqwest::Method::GET, url).headers(headers);

    let response = request.send().await?;
    let body = response.text().await?;

    Ok(body)
}

pub fn undefinite_pause() {
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}

pub fn long_pause() {
    thread::sleep(Duration::from_millis(
        rand::thread_rng().gen_range(2000..3000),
    ));
}

pub fn medium_pause() {
    thread::sleep(Duration::from_millis(
        rand::thread_rng().gen_range(1000..2000),
    ));
}

pub fn short_pause() {
    thread::sleep(Duration::from_millis(
        rand::thread_rng().gen_range(300..600),
    ));
}

pub async fn scroll_to_bottom(driver: &WebDriver) -> Result<(), WebDriverError> {
    driver
        .execute(
            r#"window.scrollTo({
  top: document.body.scrollHeight,
  left: 100,
  behavior: "smooth",
});"#,
            vec![],
        )
        .await?;
    short_pause();
    Ok(())
}

pub async fn scroll_into_view(
    driver: &WebDriver,
    element: &WebElement,
) -> Result<(), WebDriverError> {
    println!("Scrolling into view...");
    driver.execute(
        r#"arguments[0].scrollIntoView({ behavior: "smooth", block: "center", inline: "nearest" });
        "#, vec![element.to_json()?]
    ).await?;
    Ok(())
}

pub async fn swith_to_tab(driver: &WebDriver, num: usize) -> Result<(), WebDriverError> {
    let handles = driver.windows().await?;
    driver.switch_to_window(handles[num].clone()).await?;
    
    Ok(())
}

pub async fn wait_until_loaded(driver: &WebDriver) -> Result<(), Box<dyn Error>> {
    driver
        .find(By::XPath("/html/body"))
        .await?
        .wait_until()
        .displayed()
        .await?;

    Ok(())
}
