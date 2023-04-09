use fs2::FileExt;
use std::cmp::min;
use std::error::Error;
use std::fs::File;
use std::io::Write;

use super::modification::Release;

pub mod registry;

pub async fn download_release<F: Fn(u16)>(
    release: &Release,
    username: &str,
    token: &str,
    dir: &str,
    f: Option<F>,
) -> Result<File, Box<dyn Error>> {
    let url = format!(
        "https://mods.factorio.com{}?username={}&token={}",
        release.download_url, username, token
    );
    let client = reqwest::Client::new();
    let mut response = client.get(url).send().await?;
    let total_size = response.content_length().unwrap_or(1);
    let mut downloaded: usize = 0;
    let mut file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(format!("{}/{}", dir, release.file_name))?;

    file.lock_exclusive()?;

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
        downloaded = min(downloaded + (chunk.len() as usize), total_size as usize);
        let downloaded_percent = ((downloaded as f64 / total_size as f64) * 100.0) as u16;
        if let Some(ref f) = f {
            f(downloaded_percent);
        }
    }

    file.unlock()?;

    Ok(file)
}
