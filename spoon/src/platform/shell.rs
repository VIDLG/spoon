use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

pub fn cmd() -> Command {
    Command::new("cmd.exe")
}

pub fn powershell_command(script: &str) -> Command {
    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);
    command
}

pub fn cmd_start(program: &str, args: &[String]) -> Command {
    let mut command = cmd();
    command.arg("/C").arg("start").arg("").arg(program);
    for arg in args {
        command.arg(arg);
    }
    command
}

pub fn batch(program: impl AsRef<Path>) -> Command {
    let mut command = cmd();
    command.arg("/C").arg("call").arg(program.as_ref());
    command
}

pub fn direct(program: impl AsRef<OsStr>) -> Command {
    Command::new(program)
}
