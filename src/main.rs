use std::{
    collections::{HashSet, VecDeque},
    env::current_dir,
    fs::File,
    io::Read,
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use itertools::Itertools;
use rit::{Object, ObjectHeaders, ObjectTypes, Repository, TreeNode, TreeNodeType, TreeObject};

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
        #[arg()]
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
    LsTree {
        #[arg(id = "tree-ish")]
        hash: String,
        #[arg(short, long, action)]
        recursive: bool,
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
            println!("{:?}", object.header);
            println!("{:?}", object.header.serialize());
            // println!(
            //     "{:?}",
            //     String::from_utf8(object.header.serialize()).unwrap()
            // );
            // println!();
            // println!("{}", String::from_utf8(object.header.serialize()).unwrap());
        }
        Commands::HashObject { _type, write, path } => {
            // TODO: Handle not passing in a valid path file?
            let mut object_file = File::open(path).unwrap();
            let mut raw_file_contents: Vec<u8> = Vec::new();
            object_file.read_to_end(&mut raw_file_contents).unwrap();

            let object = Object::new(_type, raw_file_contents).unwrap();
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
                    panic!("Object parsed is not a commit, {:?}", commit);
                };
            }
            commit_graphviz.push('}');
            println!("{}", commit_graphviz);
        }
        Commands::LsTree { hash, recursive } => {
            let repo = Repository::find_worktree_root(current_dir().unwrap()).unwrap();
            let tree = if let ObjectHeaders::Tree(tree) =
                Object::read_from_sha(&repo, hash.clone()).unwrap().header
            {
                tree
            } else {
                panic!("Expect hash to lead to a tree object, {}", hash)
            };
            let tree_nodes = if recursive {
                fn recurse_tree(repo: &Repository, tree: TreeObject) -> Vec<TreeNode> {
                    tree.entries
                        .into_iter()
                        .map(|tree_node| {
                            if let TreeNodeType::Tree = &tree_node._type {
                                let mut tree = if let ObjectHeaders::Tree(tree) =
                                    Object::read_from_sha(repo, tree_node.hash.clone())
                                        .unwrap()
                                        .header
                                {
                                    tree
                                } else {
                                    panic!(
                                        "Excepct hash to lead to a tree object, {}",
                                        tree_node.hash
                                    )
                                };
                                tree.entries.iter_mut().for_each(|entry| {
                                    entry.path = [
                                        tree_node.clone().path,
                                        String::from("/"),
                                        entry.path.clone(),
                                    ]
                                    .concat();
                                });
                                recurse_tree(repo, tree)
                            } else {
                                vec![tree_node]
                            }
                        })
                        .reduce(|mut acc, mut next| {
                            acc.append(&mut next);
                            acc
                        })
                        .unwrap()
                }
                recurse_tree(&repo, tree)
            } else {
                tree.entries.into_iter().collect::<Vec<TreeNode>>()
            };
            println!(
                "{}",
                tree_nodes
                    .iter()
                    .map(|tree_node| {
                        format!(
                            "{} {} {}\t{}",
                            tree_node.mode, tree_node._type, tree_node.hash, tree_node.path
                        )
                    })
                    .join("\n")
            );
        }
    }
}
