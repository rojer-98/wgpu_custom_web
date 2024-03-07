use anyhow::Result;
use cfg_if::cfg_if;

pub async fn get_data<P: AsRef<str>>(file_name: P) -> Option<Vec<u8>> {
    let bin = load_binary(file_name.as_ref()).await;
    if let Err(e) = bin {
        panic!("{e}");
    }

    bin.ok()
}

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> Result<reqwest::Url> {
    use anyhow::anyhow;
    use reqwest::Url;

    let window = web_sys::window().ok_or(anyhow!("Web Sys windows not found"))?;
    let location = window.location();
    let mut origin = location
        .origin()
        .map_err(|_| anyhow!("Location origin not found"))?;

    if !origin.ends_with("assets") {
        origin = format!("{}/assets", origin);
    }
    let base = Url::parse(&format!("{}/", origin,)).map_err(|_| anyhow!("Url parse failed"))?;

    Ok(base.join(file_name)?)
}

async fn load_binary(file_name: &str) -> Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name)?;
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            use std::fs::read;

            let data = read(file_name)?;
        }
    }

    Ok(data)
}
