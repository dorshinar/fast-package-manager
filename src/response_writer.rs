use async_trait::async_trait;
use reqwest::Response;
use std::{error::Error, fs::File, io::Write, path::PathBuf};

use futures::StreamExt;

#[async_trait]
pub trait WriteResponse {
    async fn write_response(
        &self,
        response: Response,
        path: &PathBuf,
    ) -> Result<(), Box<dyn Error>>;
}

pub struct ResponseWriter;

impl ResponseWriter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WriteResponse for ResponseWriter {
    async fn write_response(
        &self,
        response: Response,
        path: &PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        let mut stream = response.bytes_stream();
        let mut tarball_file = File::create(path)?;

        while let Some(chunk) = stream.next().await {
            tarball_file.write_all(&chunk?)?;
        }

        Ok(())
    }
}
