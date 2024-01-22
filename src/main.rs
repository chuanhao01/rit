use std::env::current_dir;

use clap::{Parser, Subcommand};
use rit::Repository;

#[derive(Debug, Parser)]
#[command(name = "rit")]
#[command(about = "Rust git", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {},
    Clean {},
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Init {} => {
            Repository::init_worktree(current_dir().unwrap()).unwrap();
        }
        Commands::Clean {} => {
            Repository::clean_worktree(current_dir().unwrap()).unwrap();
        }
    }
}
