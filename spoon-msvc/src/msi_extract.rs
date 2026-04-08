//! MSI extraction utilities.
//!
//! Reads MSI cabinet names from MSI packages and extracts external CABs.

use std::collections::{HashMap, HashSet};
use std::io::{self, Read};
use std::path::Path;

use spoon_core::CoreError;

#[derive(Debug)]
struct FileEntry {
    file_name: String,
    component: String,
}

#[derive(Debug)]
struct MediaEntry {
    cabinet: String,
}

fn read_file_table(
    package: &mut msi::Package<std::fs::File>,
) -> std::result::Result<HashMap<String, FileEntry>, CoreError> {
    let mut map = HashMap::new();
    if !package.has_table("File") {
        return Ok(map);
    }

    let query = msi::Select::table("File").columns(&["File", "FileName", "Component_"]);
    let rows = package
        .select_rows(query)
        .map_err(|err| CoreError::Other(format!("querying MSI File table: {err}")))?;
    for row in rows {
        let file_key = row["File"].as_str().unwrap_or_default().to_string();
        let file_name = row["FileName"].as_str().unwrap_or_default().to_string();
        let component = row["Component_"].as_str().unwrap_or_default().to_string();
        if !file_key.is_empty() {
            map.insert(
                file_key,
                FileEntry {
                    file_name,
                    component,
                },
            );
        }
    }
    Ok(map)
}

fn read_component_table(
    package: &mut msi::Package<std::fs::File>,
) -> std::result::Result<HashMap<String, String>, CoreError> {
    let mut map = HashMap::new();
    if !package.has_table("Component") {
        return Ok(map);
    }

    let query = msi::Select::table("Component").columns(&["Component", "Directory_"]);
    let rows = package
        .select_rows(query)
        .map_err(|err| CoreError::Other(format!("querying MSI Component table: {err}")))?;
    for row in rows {
        let component = row["Component"].as_str().unwrap_or_default().to_string();
        let directory = row["Directory_"].as_str().unwrap_or_default().to_string();
        if !component.is_empty() {
            map.insert(component, directory);
        }
    }
    Ok(map)
}

fn read_directory_table(
    package: &mut msi::Package<std::fs::File>,
) -> std::result::Result<HashMap<String, (String, String)>, CoreError> {
    let mut map = HashMap::new();
    if !package.has_table("Directory") {
        return Ok(map);
    }

    let query =
        msi::Select::table("Directory").columns(&["Directory", "Directory_Parent", "DefaultDir"]);
    let rows = package
        .select_rows(query)
        .map_err(|err| CoreError::Other(format!("querying MSI Directory table: {err}")))?;
    for row in rows {
        let dir_id = row["Directory"].as_str().unwrap_or_default().to_string();
        let parent = row["Directory_Parent"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let default_dir = row["DefaultDir"].as_str().unwrap_or_default().to_string();
        if !dir_id.is_empty() {
            map.insert(dir_id, (parent, default_dir));
        }
    }
    Ok(map)
}

fn read_media_table(
    package: &mut msi::Package<std::fs::File>,
) -> std::result::Result<Vec<MediaEntry>, CoreError> {
    let mut entries = Vec::new();
    if !package.has_table("Media") {
        return Ok(entries);
    }

    let query = msi::Select::table("Media").columns(&["Cabinet"]);
    let rows = package
        .select_rows(query)
        .map_err(|err| CoreError::Other(format!("querying MSI Media table: {err}")))?;
    for row in rows {
        let cabinet = row["Cabinet"].as_str().unwrap_or_default().to_string();
        if !cabinet.is_empty() {
            entries.push(MediaEntry { cabinet });
        }
    }
    Ok(entries)
}

pub fn read_msi_cab_names(msi_path: &Path) -> std::result::Result<Vec<String>, CoreError> {
    let mut package = msi::open(msi_path)
        .map_err(|err| CoreError::Other(format!("opening MSI file '{}': {err}", msi_path.display())))?;
    Ok(read_media_table(&mut package)?
        .into_iter()
        .map(|entry| entry.cabinet)
        .collect())
}

fn resolve_directory_path(
    dir_id: &str,
    directory_table: &HashMap<String, (String, String)>,
    cache: &mut HashMap<String, String>,
) -> String {
    if let Some(cached) = cache.get(dir_id) {
        return cached.clone();
    }

    let mut parts = Vec::new();
    let mut current = dir_id.to_string();
    let mut visited = HashSet::new();

    loop {
        if visited.contains(&current) {
            break;
        }
        visited.insert(current.clone());

        let Some((parent, default_dir)) = directory_table.get(&current) else {
            break;
        };

        let dir_name = if let Some(pipe_pos) = default_dir.find('|') {
            &default_dir[pipe_pos + 1..]
        } else if let Some(colon_pos) = default_dir.find(':') {
            &default_dir[colon_pos + 1..]
        } else {
            default_dir.as_str()
        };

        if dir_name != "." && dir_name != "SourceDir" {
            parts.push(dir_name.to_string());
        }

        if parent.is_empty() {
            break;
        }
        current = parent.clone();
    }

    parts.reverse();
    let resolved = parts.join(std::path::MAIN_SEPARATOR_STR);
    cache.insert(dir_id.to_string(), resolved.clone());
    resolved
}

fn long_filename(filename_field: &str) -> &str {
    if let Some(pipe_pos) = filename_field.find('|') {
        &filename_field[pipe_pos + 1..]
    } else {
        filename_field
    }
}

fn extract_cab<R: Read + io::Seek>(
    reader: R,
    install_dir: &Path,
    file_table: &HashMap<String, FileEntry>,
    component_table: &HashMap<String, String>,
    directory_table: &HashMap<String, (String, String)>,
) -> std::result::Result<u32, CoreError> {
    let mut cabinet =
        cab::Cabinet::new(reader)
            .map_err(|err| CoreError::Other(format!("parsing CAB file: {err}")))?;
    let mut dir_cache = HashMap::new();
    let mut extracted = 0_u32;

    let file_names: Vec<String> = cabinet
        .folder_entries()
        .flat_map(|folder| folder.file_entries())
        .map(|entry| entry.name().to_string())
        .collect();

    for cab_file_name in &file_names {
        let (target_dir, actual_name) =
            if let Some(file_entry) = file_table.get(cab_file_name.as_str()) {
                let actual_name = long_filename(&file_entry.file_name);
                if let Some(dir_id) = component_table.get(&file_entry.component) {
                    let dir_path = resolve_directory_path(dir_id, directory_table, &mut dir_cache);
                    (dir_path, actual_name.to_string())
                } else {
                    (String::new(), actual_name.to_string())
                }
            } else {
                (String::new(), cab_file_name.clone())
            };

        let full_dir = if target_dir.is_empty() {
            install_dir.to_path_buf()
        } else {
            install_dir.join(&target_dir)
        };
        std::fs::create_dir_all(&full_dir)
            .map_err(|e| CoreError::fs("create_dir_all", &full_dir, e))?;
        let full_path = full_dir.join(&actual_name);
        if full_path.exists() {
            continue;
        }

        let mut reader = cabinet.read_file(cab_file_name)
            .map_err(|err| CoreError::Other(format!("reading '{}' from CAB: {err}", cab_file_name)))?;
        let mut out = std::fs::File::create(&full_path)
            .map_err(|e| CoreError::fs("create", &full_path, e))?;
        io::copy(&mut reader, &mut out)
            .map_err(|e| CoreError::fs("write", &full_path, e))?;
        extracted += 1;
    }

    Ok(extracted)
}

pub fn extract_msi_with_staged_cabs(
    msi_path: &Path,
    install_dir: &Path,
    cab_dir: &Path,
) -> std::result::Result<u32, CoreError> {
    let mut package = msi::open(msi_path)
        .map_err(|err| CoreError::Other(format!("opening MSI file '{}': {err}", msi_path.display())))?;
    let directory_table = read_directory_table(&mut package)?;
    let component_table = read_component_table(&mut package)?;
    let file_table = read_file_table(&mut package)?;
    let media_entries = read_media_table(&mut package)?;
    let mut extracted = 0_u32;
    let mut found_external = false;

    for media in &media_entries {
        if media.cabinet.is_empty() || media.cabinet.starts_with('#') {
            continue;
        }
        let cab_path = cab_dir.join(&media.cabinet);
        if !cab_path.exists() {
            continue;
        }
        let cab_file = std::fs::File::open(&cab_path)
            .map_err(|e| CoreError::fs("open", &cab_path, e))?;
        extracted += extract_cab(
            cab_file,
            install_dir,
            &file_table,
            &component_table,
            &directory_table,
        )
        .map_err(|err| CoreError::Other(format!("extracting staged CAB '{}': {err}", cab_path.display())))?;
        found_external = true;
    }

    if found_external {
        Ok(extracted)
    } else {
        Err(CoreError::Other(format!(
            "no staged external CABs were available for '{}'",
            msi_path.display()
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{long_filename, resolve_directory_path};

    #[test]
    fn long_filename_prefers_long_side_of_pipe_format() {
        assert_eq!(long_filename("READM~1.TXT|readme.txt"), "readme.txt");
        assert_eq!(long_filename("plain.txt"), "plain.txt");
    }

    #[test]
    fn resolve_directory_path_walks_directory_parent_chain() {
        let mut cache = HashMap::new();
        let table = HashMap::from([
            (
                "ROOT".to_string(),
                ("".to_string(), "SourceDir".to_string()),
            ),
            (
                "BIN".to_string(),
                ("ROOT".to_string(), "BIN|bin".to_string()),
            ),
            (
                "TOOLS".to_string(),
                ("BIN".to_string(), "TOOLS|tools".to_string()),
            ),
        ]);

        assert_eq!(
            resolve_directory_path("TOOLS", &table, &mut cache),
            format!("bin{}tools", std::path::MAIN_SEPARATOR)
        );
    }
}
