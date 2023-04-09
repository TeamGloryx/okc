use clap::Parser;
use crate::cli::CliSubcommand;

mod cli;
mod init;

fn main() {
    let cli = cli::Cli::parse();
    let verbosity = cli.verbosity();

    match cli.real {
        CliSubcommand::Init(args) => init::initialize(verbosity, args)
    }
}
