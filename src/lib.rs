extern crate chrono;
extern crate env_logger;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate libc;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json as json;
extern crate yansi;

extern crate env_path;

use chrono::offset::Utc;
use chrono::DateTime;
use failure::ResultExt;

use std::fs::File;
use std::path::{Path, PathBuf};

use env_path::EnvPath;

pub mod errors;
mod fs;
pub mod logger;

pub use errors::{AppError, AppErrorType};
use fs::LinkTree;
use logger::highlight;

pub type Result<T> = ::std::result::Result<T, AppError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    folders: Vec<Folder>,
}

impl ConfigFile {
    const RESTORE: &'static str = ".backup/config.json";

    pub fn load<T: AsRef<Path>>(path: T) -> Result<Self> {
        let file = Self::filepath(path)?;
        debug!("config file: {}", highlight(file.display()));

        let reader = File::open(&file).context(AppErrorType::Access(format!(
            "Unable to open {}",
            file.display()
        )))?;
        let folders = json::from_reader(reader).context(AppErrorType::JsonParse(format!(
            "Unable to parse {}",
            file.display()
        )))?;
        debug!("{:?}", folders);

        Ok(ConfigFile { folders })
    }

    pub fn backup<T: AsRef<Path>>(self, root: T) -> Result<()> {
        for folder in self {
            let mut tree = LinkTree::new(&folder, &root)?;
            fs::backup(&mut tree).context(AppErrorType::UpdateFolder(format!(
                "{}",
                root.as_ref().display()
            )))?
        }

        Ok(())
    }

    fn filepath<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
        let path = path.as_ref();

        if !path.is_dir() {
            return Err(AppError::from(AppErrorType::NotDir(
                path.display().to_string(),
            )));
        }

        let restore = path.join(Self::RESTORE);
        if !restore.is_file() {
            return Err(AppError::from(AppErrorType::PathUnexistant(
                restore.display().to_string(),
            )));
        }

        Ok(restore)
    }
}

impl IntoIterator for ConfigFile {
    type Item = Folder;
    type IntoIter = std::vec::IntoIter<Folder>;

    fn into_iter(self) -> Self::IntoIter {
        self.folders.into_iter()
    }
}

impl<'a> IntoIterator for &'a ConfigFile {
    type Item = &'a Folder;
    type IntoIter = std::slice::Iter<'a, Folder>;

    fn into_iter(self) -> Self::IntoIter {
        self.folders.iter()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    path: EnvPath,
    origin: EnvPath,
    description: String,
    modified: Option<DateTime<Utc>>, // parses from an RFC3339 valid string
}
