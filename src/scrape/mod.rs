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
use std::io::Write;
use url::Url;

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

pub async fn scroll_down(driver: &WebDriver) -> Result<(), WebDriverError> {
    driver
        .execute(
            r#"window.scrollBy({
  top: 200,
  left: 0,
  behavior: "smooth",
});"#,
            vec![],
        )
        .await?;
    Ok(())
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

pub async fn save_page(driver: &WebDriver) -> Result<(), Box<dyn Error>> {
    let page_source = driver.source().await?;
    let mut file = std::fs::File::create("page.html")?;
    file.write_all(page_source.as_bytes())?;

    Ok(())
}
