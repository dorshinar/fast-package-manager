use reqwest::Response;
use std::{error::Error, fs::File, io::Write, path::PathBuf};

use futures::StreamExt;

pub async fn write_response(response: Response, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut stream = response.bytes_stream();
    let mut tarball_file = File::create(path)?;

    while let Some(chunk) = stream.next().await {
        tarball_file.write_all(&chunk?)?;
    }

    Ok(())
}
