use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MsvcRuntimeArg {
    Managed,
    Official,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MsvcInstallerModeArg {
    Quiet,
    Passive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorModeArg {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Parser)]
#[command(name = "spoon")]
#[command(about = "Spoon workstation manager for Windows")]
pub struct Cli {
    #[arg(long)]
    pub root: Option<PathBuf>,

    #[arg(long)]
    pub verbose: bool,

    #[arg(long)]
    pub no_log: bool,

    #[arg(long, value_enum, default_value = "auto")]
    pub color: ColorModeArg,

    #[arg(long)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Status(StatusCommand),
    Doctor,
    Config {
        #[command(subcommand)]
        command: Option<ConfigSubcommand>,
    },
    Install(ScoopPackageCommand),
    Update(ScoopPackageCommand),
    Uninstall(ScoopPackageCommand),
    List,
    Search(ScoopSearchCommand),
    Info(ScoopSinglePackageCommand),
    Cat(ScoopSinglePackageCommand),
    Prefix(ScoopSinglePackageCommand),
    Bucket {
        #[command(subcommand)]
        command: ScoopBucketSubcommand,
    },
    Scoop {
        #[command(subcommand)]
        command: ScoopSubcommand,
    },
    Msvc {
        #[command(subcommand)]
        command: MsvcSubcommand,
    },
}

#[derive(Debug, Args)]
pub struct StatusCommand {
    #[arg(long)]
    pub refresh: bool,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    Path,
    Cat,
    Root(ConfigRootCommand),
    Msvc(ConfigScopeCommand),
    Python(ConfigScopeCommand),
    Git(ConfigScopeCommand),
}

#[derive(Debug, Args)]
pub struct ConfigRootCommand {
    pub value: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ConfigScopeCommand {
    pub key: Option<String>,

    pub value: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum ScoopSubcommand {
    Status,
    List,
    Search(ScoopSearchCommand),
    Info(ScoopSinglePackageCommand),
    Cat(ScoopSinglePackageCommand),
    Prefix(ScoopSinglePackageCommand),
    Install(ScoopPackageCommand),
    Update(ScoopPackageCommand),
    Uninstall(ScoopPackageCommand),
    Cache {
        #[command(subcommand)]
        command: DomainCacheSubcommand,
    },
    Bucket {
        #[command(subcommand)]
        command: ScoopBucketSubcommand,
    },
}

#[derive(Debug, Args)]
pub struct ScoopSearchCommand {
    pub query: Option<String>,
}

#[derive(Debug, Args)]
pub struct ScoopSinglePackageCommand {
    pub package: String,
}

#[derive(Debug, Args)]
pub struct ScoopPackageCommand {
    #[arg(value_delimiter = ',')]
    pub packages: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum ScoopBucketSubcommand {
    List,
    Add(ScoopBucketAddCommand),
    Update(ScoopBucketUpdateCommand),
    Remove(ScoopBucketRemoveCommand),
}

#[derive(Debug, Args)]
pub struct ScoopBucketAddCommand {
    pub name: String,

    pub source: Option<String>,

    #[arg(long, default_value = "master")]
    pub branch: String,
}

#[derive(Debug, Args)]
pub struct ScoopBucketRemoveCommand {
    pub name: String,
}

#[derive(Debug, Args)]
pub struct ScoopBucketUpdateCommand {
    #[arg(value_delimiter = ',')]
    pub names: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum DomainCacheSubcommand {
    Prune,
    Clear,
}

#[derive(Debug, Subcommand)]
pub enum MsvcSubcommand {
    Status,
    Install(MsvcRuntimeCommand),
    Update(MsvcRuntimeCommand),
    Uninstall(MsvcRuntimeCommand),
    Validate(MsvcValidateCommand),
    Cache {
        #[command(subcommand)]
        command: DomainCacheSubcommand,
    },
}

#[derive(Debug, Args)]
pub struct MsvcRuntimeCommand {
    #[arg(value_enum)]
    pub runtime: MsvcRuntimeArg,

    #[arg(long, value_enum, default_value = "quiet")]
    pub mode: MsvcInstallerModeArg,

    #[arg(long, conflicts_with = "mode")]
    pub passive: bool,

    #[arg(long, conflicts_with_all = ["mode", "passive"])]
    pub quiet: bool,
}

#[derive(Debug, Args)]
pub struct MsvcValidateCommand {
    #[arg(value_enum)]
    pub runtime: Option<MsvcRuntimeArg>,
}
