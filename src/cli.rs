use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand, Args, ArgAction};
use okc::version::MinecraftVersion;

#[derive(Parser)]
#[command(author = "gh:TeamGloryx")]
#[command(version = okc::VERSION)]
#[command(about = "OKCraft - A tool for quick creation and management of Minecraft servers from the command line.")]
pub struct Cli {
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
    #[command(subcommand)]
    pub real: CliSubcommand
}

impl Cli {
    pub fn verbosity(&self) -> Verbosity {
        match self.verbose {
            0 => Verbosity::None,
            1 => Verbosity::Verbose,
            _ => Verbosity::VeryVerbose
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Verbosity {
    None,
    Verbose,
    VeryVerbose
}

#[derive(Subcommand)]
pub enum CliSubcommand {
    Init(InitArgs)
}

#[derive(Args)]
pub struct InitArgs {
    dir: PathBuf,
    pub version: MinecraftVersion
}

impl InitArgs {
    #[must_use]
    pub fn dir(&self) -> &Path {
        self.dir.as_path()
    }
}
