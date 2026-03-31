use std::path::{Path, PathBuf};
use crate::Result;

pub trait SystemPort {
    fn home_dir(&self) -> PathBuf;
    fn ensure_user_path_entry(&self, path: &Path) -> Result<()>;
    fn ensure_process_path_entry(&self, path: &Path);
    fn remove_user_path_entry(&self, path: &Path) -> Result<()>;
    fn remove_process_path_entry(&self, path: &Path);
}
