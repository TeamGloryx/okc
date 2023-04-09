use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand, Args, ArgAction};
use okc::flavor::MinecraftVersion;

#[derive(Parser)]
pub struct Cli {
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
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
    version: MinecraftVersion
}

impl InitArgs {
    #[must_use]
    pub fn dir(&self) -> &Path {
        self.dir.as_path()
    }
}
