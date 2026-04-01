use crate::{msiexec_path, windows_system_tool};

#[test]
fn windows_system_tool_uses_windir_when_present() {
    unsafe {
        std::env::set_var("WINDIR", r"D:\CustomWindows");
    }
    assert_eq!(
        windows_system_tool("tool.exe"),
        std::path::PathBuf::from(r"D:\CustomWindows\System32\tool.exe")
    );
    unsafe {
        std::env::remove_var("WINDIR");
    }
}

#[test]
fn msiexec_path_defaults_to_windows_system32() {
    unsafe {
        std::env::remove_var("WINDIR");
    }
    assert_eq!(
        msiexec_path(),
        std::path::PathBuf::from(r"C:\Windows\System32\msiexec.exe")
    );
}
