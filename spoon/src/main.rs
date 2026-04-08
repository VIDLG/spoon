use std::path::PathBuf;

use clap::Parser;
use spoon::cli::Cli;
use spoon::{cli, config, editor, launcher, logger, status, tui};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let cli = Cli::parse();
    let json_mode = cli.json;
    let result = real_main(cli).await;
    logger::flush_buffered_stdout();
    if let Err(error) = result {
        if json_mode {
            spoon::cli::print_json_value(&spoon::cli::json_error(&error));
        } else {
            eprintln!("Error: {error:#}");
        }
        std::process::exit(1);
    }
}

async fn real_main(cli: Cli) -> anyhow::Result<()> {
    cli::set_color_mode(cli.color);
    logger::init(logger::LoggerSettings::standard(cli.verbose));
    logger::app_start(format!("{:?}", cli.command));
    let test_mode = std::env::var_os("SPOON_TEST_MODE").is_some();
    if let Some(home) = std::env::var_os("SPOON_TEST_HOME") {
        config::set_home_override(PathBuf::from(home));
    }
    if test_mode {
        config::enable_test_mode();
        editor::enable_test_mode();
        launcher::enable_test_mode();
    } else {
        status::refresh_process_env_from_registry()?;
    }

    let install_root = cli.root.clone().or_else(config::configured_tool_root);

    if cli.command.is_none() {
        return tokio::task::spawn_blocking(move || tui::run_tui(install_root, PathBuf::from(".")))
            .await
            .map_err(|err| anyhow::anyhow!("failed to join TUI task: {err}"))?;
    }

    cli::run_command(
        cli.command.unwrap(),
        install_root.as_deref(),
        cli.root,
        cli.json,
    )
    .await?;

    Ok(())
}
