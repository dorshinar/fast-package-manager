use std::{
    error::Error,
    fs::{self, File},
    path::PathBuf,
};

use flate2::read::GzDecoder;
use tar::Archive;

pub trait Extract {
    fn extract_tar_gz(&self, path: &PathBuf, dest: &PathBuf) -> Result<(), Box<dyn Error>>;
}

pub struct NpmTarExtractor;

impl NpmTarExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl Extract for NpmTarExtractor {
    fn extract_tar_gz(&self, path: &PathBuf, dest: &PathBuf) -> Result<(), Box<dyn Error>> {
        let tar_gz_file = File::open(&path)?;
        let tar_file = GzDecoder::new(tar_gz_file);

        let mut archive = Archive::new(tar_file);

        fs::create_dir_all(&dest)?;

        for file in archive.entries().unwrap() {
            let mut file = file.unwrap();

            let file_path = file.path().unwrap();
            let file_path = file_path.strip_prefix("package")?;
            file.unpack(dest.join(file_path))?;
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::{fs, io::Write, path::Path};

//     use flate2::{read::GzEncoder, Compression};

//     use super::*;

//     #[test]
//     fn extracts_tar_gz() {
//         // let test_folder = "./extract_tests/extract_tar_gz";
//         // fs::create_dir_all(test_folder).expect("failed to create test folder");

//         // let mut file = File::create(Path::new(test_folder).join("test_file"))
//         //     .expect("failed to create test file");
//         // file.write_all(b"file_content")
//         //     .expect("failed to write to test file");

//         // let tar_file = File::create("extract.tar.gz").unwrap();
//         // let encoded = GzEncoder::new(tar_file, Compression::fast());
//         // let mut tar = tar::Builder::new(encoded);
//         // let mut tar_path = "package".to_owned();
//         // tar_path.push_str(test_folder);
//         // tar.append_dir_all(tar_path, test_folder).unwrap();
//         // tar.finish().unwrap();

//         // drop(file);
//         // drop(tar);

//         // fs::remove_dir_all("./extract_tests").unwrap();

//         // let extractor = TarExtractor::new();
//         // extractor
//         //     .extract_tar_gz(&PathBuf::from("extract.tar.gz"), &PathBuf::from("."))
//         //     .unwrap();

//         // let file_content = fs::read_to_string(Path::new(test_folder).join("test_file")).unwrap();
//         // assert_eq!(file_content, "file_content");
//     }
// }
