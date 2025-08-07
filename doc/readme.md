# hypochlorite docs

## Naming

hypochlorite is a common bleach for "scraping rusts". (Dark humour, lol).

## Thirty four

Thirty-four is a library for controlling the chrome driver. 
To use it, you need to download the chrome driver and chrome browser. Their version must match.

To launch Thirty-four 

```rust
// you need to run chrome driver at port 9515 (default port) 
let caps = DesiredCapabilities::chrome();
// get the handler
let driver = WebDriver::new("http://localhost:9515", caps)
    .await
    .unwrap();
```

In this library, running the chrome driver and initialise the handler are combined into the function .
However, ypou still need to download the chrom driver and place it in root dir.
```rust
// main.rs
let _kill_guard = web_driver::KillChildGuard;
let driver = web_driver::initialize_driver().await?;
```

Here are the commonly used function
```rust
// Open a new tab.
driver.new_tab().await?;
// Get window handles and switch to the new tab.
let handles = driver.windows().await?;
driver.switch_to_window(handles[1].clone()).await?;
// We are now controlling the new tab.
driver.goto("https://www.rust-lang.org").await?;

let elem = driver.find(By::Id("my-element-id")).await?;
let child_elems = elem.find_all(By::Tag("button")).await?;
for child_elem in child_elems {
    assert_eq!(child_elem.tag_name().await?, "button");
}
// find by XPath
if let Ok(popup_menu_ok_button) = driver
    .find(By::XPath("/html/body/dialog[1]/div[2]/button[2]"))
    .await
{
    popup_menu_ok_button.wait_until().clickable().await?;
    popup_menu_ok_button.click().await?;
    return Ok(());
}
// execute javascript. Element is a WebElement.
driver.execute( 
    r#"arguments[0].scrollIntoView({ behavior: "smooth", block: "center", inline: "nearest" });
    "#, vec![element.to_json()?]
).await?;
```
