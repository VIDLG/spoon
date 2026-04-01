use std::path::PathBuf;

pub fn windows_system_tool(executable: &str) -> PathBuf {
    let windir = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    windir.join("System32").join(executable)
}

pub fn msiexec_path() -> PathBuf {
    windows_system_tool("msiexec.exe")
}
