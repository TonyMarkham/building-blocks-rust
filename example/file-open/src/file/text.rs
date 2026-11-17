use crate::{file::open_at, FileError, FileResult};

use std::{panic::Location,  path::Path};
use error_location::ErrorLocation;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

#[track_caller]
pub(crate) fn read_text(path: impl AsRef<Path>) -> impl Future<Output = FileResult<String>> {
    let location = ErrorLocation::from(Location::caller());
    let path = path.as_ref().to_path_buf();
    async move {
        let mut file = open_at(path.clone(), location).await?;
        let mut contents = String::new();
        match file.read_to_string(&mut contents).await {
            Ok(_) => Ok(contents),
            Err(source) => Err(FileError::from_io(source, path, location)),
        }
    }
}

#[track_caller]
pub(crate) fn read_lines(path: impl AsRef<Path>) -> impl Future<Output = FileResult<Vec<String>>> {
    let location = ErrorLocation::from(Location::caller());
    let path = path.as_ref().to_path_buf();
    async move {
        let file = open_at(path.clone(), location).await?;
        let mut reader = BufReader::new(file).lines();
        let mut lines = Vec::new();
        loop {
            match reader.next_line().await {
                Ok(Some(line)) => lines.push(line),
                Ok(None) => return Ok(lines),
                Err(source) => return Err(FileError::from_io(source, path, location)),
            }
        }
    }
}