use chrono::Utc;
use failure::ResultExt;
use log::{info, log};
use std::io;
use std::path::Path;

use crate::{AppError, AppResult, ErrorType};
use artid::ops::core::{CopyAction, Route};
use artid::ops::restore::{self, Options};
use artid::prelude::*;
use logger::pathlight;

pub fn restore(
    run: bool,
    overwrite: bool,
    path: &Path,
    folder: &Option<String>,
    point: &Option<usize>,
) -> AppResult<()> {
    info!("Starting restore of the contents in {}", pathlight(path));

    let options = match point.to_owned() {
        Some(value) => Options::with_point(overwrite, value),
        None => Options::new(overwrite),
    };

    let mut config = ConfigFile::load(path)?;

    match folder {
        Some(ref value) => {
            let mut folder = get_folder(&mut config, value)?;
            let model = restore::restore(&mut folder, options).context(ErrorType::Operative)?;
            operate(run, model)?;
        }

        None => {
            let model = restore::restore(&mut config, options).context(ErrorType::Operative)?;
            operate(run, model)?;
        }
    }

    config.save()?;
    Ok(())
}

fn get_folder<'a, P>(config: &'a mut ConfigFile<P>, value: &str) -> AppResult<FileSystemFolder<'a>>
where
    P: AsRef<Path> + ::std::fmt::Debug,
{
    config.get_folder(value).ok_or_else(|| {
        AppError::from(ErrorType::BadArgument(
            value.to_string(),
            "--folder".to_string(),
        ))
    })
}

fn operate<M>(run: bool, model: M) -> AppResult<()>
where
    M: Model<Action = restore::Action, Error = io::Error>,
{
    if run {
        model.run().context(ErrorType::Operative)?;
        info!("Restore performed successfully");
    } else {
        model.log(&|action| {
            if let CopyAction::CopyFile { ref src, ref dst } = action {
                info!(
                    "sync {} -> {}",
                    pathlight(src.path()),
                    pathlight(dst.path())
                );
            }
        });
    }

    Ok(())
}
