use std::{
    error,
    fs::{self},
    io::{Cursor, ErrorKind},
    path::Path,
};

use async_compression::tokio::bufread::GzipDecoder;
use futures::TryStreamExt;
use tar::Archive;
use tokio::io::{self, AsyncReadExt};
use tokio_util::compat::FuturesAsyncReadCompatExt;

use crate::{http::get_package_tar, npm::UrlString, STORE_FOLDER};

pub async fn download_package_to_store(
    package_name: String,
    tar: UrlString,
) -> Result<(), Box<dyn error::Error>> {
    let tar_content = get_package_tar(&tar).await?;

    let deps_dest = Path::new(STORE_FOLDER).join(&package_name);

    let tgz = GzipDecoder::new(
        tar_content
            .bytes_stream()
            .map_err(|e| io::Error::new(ErrorKind::Other, e))
            .into_async_read()
            .compat(),
    );

    let mut buf_reader = tokio::io::BufReader::new(tgz);
    let mut content = Vec::new();
    buf_reader.read_to_end(&mut content).await?;
    let mut archive = Archive::new(Cursor::new(content));

    fs::create_dir_all(&deps_dest)?;

    for file in archive.entries().unwrap() {
        let mut file = file.unwrap();

        let file_path = file.path().unwrap();
        let file_path = file_path.strip_prefix("package")?;
        file.unpack(deps_dest.join(file_path))?;
    }

    Ok(())
}
