use std::{
    collections::{HashSet, VecDeque},
    env::current_dir,
    fs::{remove_dir_all, File},
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use itertools::Itertools;
use rit::{
    create_dir, create_path, resolve_ref, Object, ObjectHeaders, ObjectTypes, Repository, TreeNode,
    TreeNodeType, TreeObject, GIT_DIR_PATH, RIT_DIR_PATH,
};

#[derive(Debug, Parser)]
#[command(name = "rit")]
#[command(about = "Rust git", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, action)]
    git_dir: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {},
    Clean {},
    CatFile {
        #[arg()]
        _type: ObjectTypes,
        #[arg(id = "OBJECT")]
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
        #[arg(id = "COMMIT")]
        hash: String,
    },
    LsTree {
        #[arg(id = "tree-ish")]
        hash: String,
        #[arg(short, long, action)]
        recursive: bool,
    },
    Checkout {
        #[arg(id = "COMMIT")]
        hash: String,
        #[arg()]
        /// Directory to checkout in
        path: PathBuf,
        #[arg(short, long, action)]
        _override: bool,
    },
    ShowRef {
        #[arg(long)]
        head: bool,
    },
    Tag {
        // Creates a new tag object
        #[arg(id = "a", short)]
        annotate: bool,
    },
}

fn main() {
    let args = Cli::parse();
    let git_dir_path = if args.git_dir {
        GIT_DIR_PATH
    } else {
        RIT_DIR_PATH
    };

    match args.command {
        Commands::Init {} => {
            Repository::init_worktree(current_dir().unwrap(), git_dir_path).unwrap();
        }
        Commands::Clean {} => {
            Repository::clean_worktree(current_dir().unwrap(), git_dir_path).unwrap();
        }
        Commands::CatFile {
            hash: object,
            _type,
        } => {
            let object_identifier = object;
            let repo =
                Repository::find_worktree_root(current_dir().unwrap(), git_dir_path).unwrap();
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
                let repo =
                    Repository::find_worktree_root(current_dir().unwrap(), git_dir_path).unwrap();
                object.write_to_repo(&repo).unwrap()
            };
            println!("{}", hash);
        }
        Commands::Log { hash } => {
            // Only takes in full commit hashes for now
            // TODO:Convert the given hash value (As it can be in short form)
            // println!("DEBUG, {}", hash);
            let repo =
                Repository::find_worktree_root(current_dir().unwrap(), git_dir_path).unwrap();
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
            let repo =
                Repository::find_worktree_root(current_dir().unwrap(), git_dir_path).unwrap();
            let tree = if let ObjectHeaders::Tree(tree) =
                Object::read_from_sha(&repo, hash.clone()).unwrap().header
            {
                tree
            } else {
                panic!("Expect hash to lead to a tree object, {}", hash)
            };
            let tree_nodes = if recursive {
                fn recurse_tree(
                    repo: &Repository,
                    base_path: &str,
                    cur_tree: TreeObject,
                ) -> Vec<(TreeNode, String)> {
                    cur_tree
                        .entries
                        .into_iter()
                        .map(|tree_node| {
                            if let TreeNodeType::Tree = &tree_node._type {
                                let tree = if let ObjectHeaders::Tree(tree) =
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
                                // NOTE: since we start with "", "/" is added after the path to have old_dir/new_dir pattern
                                recurse_tree(
                                    repo,
                                    &[base_path, &tree_node.path, "/"].concat(),
                                    tree,
                                )
                            } else {
                                vec![(tree_node, base_path.to_owned())]
                            }
                        })
                        .reduce(|mut acc, mut next| {
                            acc.append(&mut next);
                            acc
                        })
                        .unwrap()
                }
                recurse_tree(&repo, "", tree)
            } else {
                tree.entries
                    .into_iter()
                    .map(|tree_node| (tree_node, String::from("")))
                    .collect::<Vec<(TreeNode, String)>>()
            };
            println!(
                "{}",
                tree_nodes
                    .iter()
                    .map(|(tree_node, base_path)| {
                        format!(
                            "{} {} {}\t{}{}",
                            tree_node.mode,
                            tree_node._type,
                            tree_node.hash,
                            base_path,
                            tree_node.path
                        )
                    })
                    .join("\n")
            );
        }
        Commands::Checkout {
            hash,
            path,
            _override,
        } => {
            let repo =
                Repository::find_worktree_root(current_dir().unwrap(), git_dir_path).unwrap();
            let object = Object::read_from_sha(&repo, hash.clone()).unwrap();
            let tree_hash = if let ObjectHeaders::Commit { fields, .. } = object.header {
                fields.get("tree").unwrap_or_else(|| {
                    panic!("Commit has no tree(hash) field, fields: {:?}", fields)
                })[0]
                    .to_owned()
            } else {
                panic!("Given hash, {}, is not a commit, {:?}", hash, object);
            };
            let tree = if let ObjectHeaders::Tree(tree) =
                Object::read_from_sha(&repo, tree_hash.clone())
                    .unwrap()
                    .header
            {
                tree
            } else {
                panic!("Excepct hash to lead to a tree object, {}", tree_hash);
            };

            if _override {
                if let Err(e) = remove_dir_all(path.clone()) {
                    match e.kind() {
                        ErrorKind::NotFound => {}
                        _ => panic!("Unable to remove and override {:?}, {:?}", path.clone(), e),
                    }
                }
            }
            create_dir(&path).unwrap();

            fn recurse_tree_checkout_blob(
                repo: &Repository,
                checkout_path: &Path,
                base_path: &str,
                cur_tree: TreeObject,
            ) {
                for cur_entry in &cur_tree.entries {
                    match cur_entry._type {
                        TreeNodeType::Blob => {
                            let blob_contents = if let ObjectHeaders::Blob { data } =
                                Object::read_from_sha(repo, cur_entry.hash.to_owned())
                                    .unwrap()
                                    .header
                            {
                                data
                            } else {
                                panic!("Expected blob from tree node, {:?}", cur_entry)
                            };
                            let blob_path = create_path(
                                checkout_path,
                                vec![base_path.to_owned(), cur_entry.path.to_owned()],
                            );
                            println!("{:?}", blob_path);
                            let mut blob_file = File::create(blob_path).unwrap();
                            blob_file.write_all(&blob_contents).unwrap();
                        }
                        TreeNodeType::Tree => {
                            let nested_tree_path = create_path(
                                checkout_path,
                                vec![base_path.to_owned(), cur_entry.path.to_owned()],
                            );
                            create_dir(&nested_tree_path).unwrap();
                            let nested_tree = if let ObjectHeaders::Tree(tree) =
                                Object::read_from_sha(repo, cur_entry.hash.clone())
                                    .unwrap()
                                    .header
                            {
                                tree
                            } else {
                                panic!("Excepct hash to lead to a tree object, {}", cur_entry.hash)
                            };
                            // NOTE: since we start with "", "/" is added after the path to have old_dir/new_dir pattern
                            recurse_tree_checkout_blob(
                                repo,
                                checkout_path,
                                &[base_path, &cur_entry.path, "/"].concat(),
                                nested_tree,
                            );
                        }
                        _ => {
                            // Submodules and symbolic links are not implemented
                        }
                    }
                }
            }
            recurse_tree_checkout_blob(&repo, &path, "", tree);
        }
        Commands::ShowRef { head } => {
            use walkdir::WalkDir;
            let repo =
                Repository::find_worktree_root(current_dir().unwrap(), git_dir_path).unwrap();
            let tags_not_in_refs = vec![create_path(&repo.gitdir, vec![String::from("HEAD")])];
            let tags_in_refs = WalkDir::new(create_path(&repo.gitdir, vec![String::from("refs")]))
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_file())
                .map(|entry| entry.path().to_owned())
                .collect::<Vec<PathBuf>>();
            for path in if head {
                [tags_not_in_refs, tags_in_refs].concat()
            } else {
                tags_in_refs
            } {
                let _ref = resolve_ref(&repo, &path).unwrap();
                println!(
                    "{} {}",
                    _ref,
                    path.to_str()
                        .unwrap()
                        .strip_prefix(repo.gitdir.to_str().unwrap())
                        .unwrap()
                        .strip_prefix('/')
                        .unwrap()
                );
            }
        }
        Commands::Tag { annotate } => {}
    }
}
