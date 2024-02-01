use std::{env::current_dir, fs::File, io::Read, path::PathBuf};

use clap::{Parser, Subcommand};
use rit::{Object, ObjectTypes, Repository};

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
    CatFile {
        #[arg(short, long)]
        _type: ObjectTypes,
        #[arg()]
        object: String,
    },
    /// Computes the object hash and optionally creates a blob from a file
    HashObject {
        #[arg(short, long)]
        _type: ObjectTypes,
        /// Actually writes the object into the database
        #[arg(short, long, action)]
        write: bool,
        /// Read the object from path
        path: PathBuf,
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
        Commands::CatFile { object, _type } => {
            let object_identifier = object;
            let repo = Repository::find_worktree_root(current_dir().unwrap()).unwrap();
            let object = Object::read_from_sha(repo, object_identifier).unwrap();
            println!("{:?}", object.header.serialize());
            println!(
                "{:?}",
                String::from_utf8(object.header.serialize()).unwrap()
            );
        }
        Commands::HashObject { _type, write, path } => {
            // TODO: Handle not passing in a valid path file?
            let mut object_file = File::open(path).unwrap();
            let mut raw_file_contents: Vec<u8> = Vec::new();
            object_file.read_to_end(&mut raw_file_contents).unwrap();

            let object = Object::new(_type, raw_file_contents);
            let hash = if !write {
                object.calculate_hash().unwrap()
            } else {
                let repo = Repository::find_worktree_root(current_dir().unwrap()).unwrap();
                object.write_to_repo(repo).unwrap()
            };
            println!("{}", hash);
        }
    }
}
