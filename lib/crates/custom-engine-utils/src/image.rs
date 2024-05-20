use anyhow::Result;
use cfg_if::cfg_if;

pub fn get_data<P: AsRef<str>>(file_name: P) -> Option<Vec<u8>> {
    let bin = load_binary(file_name.as_ref());
    if let Err(e) = bin {
        panic!("{e}");
    }

    bin.ok()
}

pub fn get_string<P: AsRef<str>>(file_name: P) -> Option<String> {
    let bin = get_data(file_name)?;
    let s = String::from_utf8(bin);

    if let Err(e) = s {
        panic!("{e}");
    }

    s.ok()
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

    let base =
        Url::parse(&format!("{origin}/{file_name}")).map_err(|_| anyhow!("Url parse failed"))?;

    Ok(base)
}

fn load_binary(file_name: &str) -> Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use pollster::block_on;

            let url = format_url(file_name)?;
            let req  = block_on(async {
                reqwest::get(url).await
            })?;
            let bytes = block_on(async {
                req.bytes().await
            })?;

            let data = bytes.to_vec();

        } else {
            use std::fs::read;

            let data = read(file_name)?;
        }
    }

    Ok(data)
}
