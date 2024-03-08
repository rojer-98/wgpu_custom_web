use anyhow::Result;
use cfg_if::cfg_if;

pub async fn get_data<P: AsRef<str>>(file_name: P) -> Option<Vec<u8>> {
    let bin = load_binary(file_name.as_ref()).await;
    if let Err(e) = bin {
        panic!("{e}");
    }

    bin.ok()
}

pub async fn get_string<P: AsRef<str>>(file_name: P) -> Option<String> {
    let bin = get_data(file_name).await?;
    let s = String::from_utf8(bin);

    if let Err(e) = s {
        panic!("{e}");
    }

    s.ok()
}

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> Result<reqwest::Url> {
    use anyhow::anyhow;
    use log::info;
    use reqwest::Url;

    let window = web_sys::window().ok_or(anyhow!("Web Sys windows not found"))?;
    let location = window.location();
    let mut origin = location
        .origin()
        .map_err(|_| anyhow!("Location origin not found"))?;

    let base =
        Url::parse(&format!("{origin}/{file_name}")).map_err(|_| anyhow!("Url parse failed"))?;

    Ok(base)
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
