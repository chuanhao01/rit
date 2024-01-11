use std::{
    collections::HashMap,
    env,
    fs::{create_dir, File},
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use configparser::ini::Ini;
use homedir::get_my_home;

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
}

struct Repository {
    worktree: PathBuf,
    gitdir: PathBuf,
}
impl Repository {
    pub fn worktree_builder(worktree: PathBuf) -> Self {
        if !worktree.exists() {
            panic!("Not able to access worktree directory: {:?}", worktree);
        }

        let mut gitdir = worktree.clone();
        gitdir.push(".git");

        Self::create_dir_ignore_exists(&gitdir, Vec::new()).unwrap();

        Self { worktree, gitdir }
    }
    fn create_dir_ignore_exists(base_path: &PathBuf, paths: Vec<String>) -> Result<(), String> {
        let mut new_path = base_path.clone();
        for path in paths {
            new_path.push(path);
        }
        if let Err(e) = create_dir(new_path.clone()) {
            if !new_path.exists() {
                return Err(format!("Not able to create dir, {:?}: {}", new_path, e));
            }
        }
        Ok(())
    }
    fn create_file_ignore_exists(base_path: &PathBuf, paths: Vec<String>) -> Result<(), String> {
        let mut new_path = base_path.clone();
        for path in paths {
            new_path.push(path);
        }
        if let Ok(_) = File::open(new_path) {
            Ok(())
        } else {
            File::create(new_path)
                .map(|| Ok(()))
                .map_err(|e| format!("Not able to create file"))
        }
    }
    // fn from_worktree(worktree: PathBuf) -> Result<Self, String> {
    //     // Get system config
    //     let system_config = Config::get_system_config();

    //     if !worktree.exists() {
    //         return Err(String::from("Not able to access current directory"));
    //     }

    //     let mut gitdir = worktree.clone();
    //     gitdir.push(".git");
    //     if !gitdir.exists() {
    //         return Err(String::from(".git directory does not exsists"));
    //     }

    //     let mut repo_ini_path = gitdir.clone();
    //     repo_ini_path.push("config");
    //     let mut repo_ini = Ini::new();
    //     repo_ini
    //         .load(repo_ini_path)
    //         .map_err(|e| format!("Error loading .git directory config: {}", e))?;
    //     let repo_config = Config::from_ini(repo_ini);

    //     Ok(Self {
    //         worktree,
    //         gitdir,
    //         config: system_config.merge(&repo_config),
    //     })
    // }
}

struct Config {
    core: CoreConfig,
}
impl Config {
    fn from_ini(ini: Ini) -> Self {
        let mut config = Config::default();
        if let Some(ini_config) = ini.get_map() {
            if let Some(hashmap) = ini_config.get("core") {
                config.core = CoreConfig::from_hashmap(hashmap.clone());
            }
        };
        config
    }
    /// Helper method to get the user's system wide config, returns default if it fails to find it
    fn get_system_config() -> Self {
        if let Ok(Some(user_home)) = get_my_home() {
            let mut ini = Ini::new();
            if let Err(e) = ini.load(user_home.as_path()) {
                return Self::default();
            };
            Self::from_ini(ini)
        } else {
            Self::default()
        }
    }
    fn merge(&self, other: &Self) -> Self {
        Self {
            core: other.core.clone(),
            ..*self
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
        }
    }
}

#[derive(Clone)]
struct CoreConfig {
    repo: String,
}
impl CoreConfig {
    fn from_hashmap(hashmap: HashMap<String, Option<String>>) -> Self {
        let mut config = Self::default();
        if let Some(config_val) = hashmap.get("repo") {
            if let Some(val) = config_val {
                config.repo = val.to_string();
            }
        }
        config
    }
}
impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            repo: String::from("nothing"),
        }
    }
}

fn main() {
    let args = Cli::parse();

    println!("{:?}", args);
}
