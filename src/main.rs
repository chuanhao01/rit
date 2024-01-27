use std::env::current_dir;

use clap::{Parser, Subcommand};
use rit::{Object, ObjectHeaders, Repository};

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
    TestWrite {},
    TestRead {
        #[arg(long)]
        hash: String,
    },
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
        Commands::TestRead { hash } => {
            let repo = Repository::find_worktree_root(current_dir().unwrap()).unwrap();
            let object = Object::read_from_sha(repo, hash).unwrap();
            println!("{:?}", object.data);
        }
        Commands::TestWrite {} => {
            let repo = Repository::find_worktree_root(current_dir().unwrap()).unwrap();
            let data = b"hello world".to_ascii_lowercase();
            let object = Object {
                data,
                header: ObjectHeaders::Commit,
            };
            let hash = object.write_to_repo(repo).unwrap();
            // Object::write_to_repo(Object::Commit, repo, data.to_ascii_lowercase()).unwrap();
            println!("{}", hash);
        }
    }
}
