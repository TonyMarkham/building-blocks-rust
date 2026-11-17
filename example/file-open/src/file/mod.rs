pub(crate) mod text;
pub(crate) mod png;

// -------------------------------------------------------------------------- //

use crate::{FileError, FileResult};

use error_location::ErrorLocation;
use std::{
    panic::Location,
    path::{Path, PathBuf},
};
use tokio::{fs::File, io::AsyncReadExt};

#[track_caller]
pub(crate) fn open(path: impl AsRef<Path>) -> impl Future<Output = FileResult<File>> {
    let location = ErrorLocation::from(Location::caller());
    open_at(path.as_ref().to_path_buf(), location)
}

/// Read at most `len` bytes from the start of the file.
/// Returns fewer bytes if the file is shorter.
#[track_caller]
pub(crate) fn read_prefix(path: impl AsRef<Path>, len: u64) -> impl Future<Output = FileResult<Vec<u8>>> {
    let location = ErrorLocation::from(Location::caller());
    let path = path.as_ref().to_path_buf();
    async move {
        let file = open_at(path.clone(), location).await?;
        let mut bytes = Vec::new();
        match file.take(len).read_to_end(&mut bytes).await {
            Ok(_) => Ok(bytes),
            Err(source) => Err(FileError::from_io(source, path, location)),
        }
    }
}

async fn open_at(path: PathBuf, location: ErrorLocation) -> FileResult<File> {
    match File::open(&path).await {
        Ok(file) => Ok(file),
        Err(source) => Err(FileError::from_io(source, path, location)),
    }
}
