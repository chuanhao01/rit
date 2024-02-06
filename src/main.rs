use std::{
    collections::{HashSet, VecDeque},
    env::current_dir,
    fmt::format,
    fs::File,
    io::Read,
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use rit::{Object, ObjectHeaders, ObjectTypes, Repository};

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
        #[arg(id = "object")]
        hash: String,
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
    Log {
        // Hash of commit to start at
        #[arg(id = "commit", default_value = "HEAD")]
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
        Commands::CatFile {
            hash: object,
            _type,
        } => {
            let object_identifier = object;
            let repo = Repository::find_worktree_root(current_dir().unwrap()).unwrap();
            let object = Object::read_from_sha(&repo, object_identifier).unwrap();
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
                object.write_to_repo(&repo).unwrap()
            };
            println!("{}", hash);
        }
        Commands::Log { hash } => {
            // Only takes in full commit hashes for now
            // TODO:Convert the given hash value (As it can be in short form)
            // println!("DEBUG, {}", hash);
            let repo = Repository::find_worktree_root(current_dir().unwrap()).unwrap();
            let mut commits_queue: VecDeque<String> = VecDeque::from([hash]);
            let mut commit_graphviz = String::from("digraph rit{\nnode[shape=rect]\n");
            let mut seen_hashes: HashSet<String> = HashSet::new();
            while !commits_queue.is_empty() {
                let current_commit_hash = commits_queue.pop_front().unwrap();
                if seen_hashes.contains(&current_commit_hash) {
                    continue;
                }

                let commit = Object::read_from_sha(&repo, current_commit_hash.clone()).unwrap();
                if let ObjectHeaders::Commit {
                    fields,
                    message,
                    order: _,
                } = commit.header
                {
                    let mut message = message.trim().to_owned();
                    message = message.replace('\\', "\\\\");
                    message = message.replace('\"', "\\\"");
                    let commit_label = format!(
                        "c_{} [label=\"{}; {}\"]",
                        current_commit_hash,
                        String::from_utf8(current_commit_hash.as_bytes()[..7].to_ascii_lowercase())
                            .unwrap(),
                        message
                    );
                    commit_graphviz.push_str(&commit_label);
                    commit_graphviz.push('\n');
                    seen_hashes.insert(current_commit_hash.clone());

                    if let Some(parents) = fields.get("parent") {
                        for parent in parents {
                            let parent_edge = format!("c_{} -> c_{}", current_commit_hash, parent);
                            commit_graphviz.push_str(&parent_edge);
                            commit_graphviz.push('\n');
                            commits_queue.push_back(parent.to_owned());
                        }
                    }
                } else {
                    panic!("Commit failed");
                };
            }
            commit_graphviz.push('}');
            println!("{}", commit_graphviz);
        }
    }
}
