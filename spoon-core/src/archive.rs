use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

use crate::{CoreError, Result};

pub fn extract_zip_archive_sync(archive_path: &Path, destination: &Path) -> Result<()> {
    let file =
        File::open(archive_path).map_err(|err| CoreError::fs("open", archive_path, err))?;
    let mut archive = ZipArchive::new(file).map_err(|err| {
        CoreError::external(format!("invalid zip {}", archive_path.display()), err)
    })?;
    std::fs::create_dir_all(destination)
        .map_err(|err| CoreError::fs("create", destination, err))?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|err| CoreError::external("invalid zip entry", err))?;
        let Some(name) = entry.enclosed_name().map(PathBuf::from) else {
            continue;
        };
        let output_path = destination.join(name);
        if entry.is_dir() {
            std::fs::create_dir_all(&output_path)
                .map_err(|err| CoreError::fs("create", &output_path, err))?;
            continue;
        }
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| CoreError::fs("create", parent, err))?;
        }
        let mut output = File::create(&output_path)
            .map_err(|err| CoreError::fs("create", &output_path, err))?;
        io::copy(&mut entry, &mut output).map_err(|err| {
            CoreError::Other(format!(
                "failed to extract {}: {err}",
                output_path.display()
            ))
        })?;
    }
    Ok(())
}
