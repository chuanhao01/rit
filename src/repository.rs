use std::{fs::create_dir, path::PathBuf};

use configparser::ini::Ini;

use crate::{create_path, Config};

struct Repository {
    worktree: PathBuf,
    gitdir: PathBuf,
    config: Config,
}
impl Repository {
    pub fn init_worktree(worktree: PathBuf) -> Result<Self, String> {
        let gitdir = create_path(&worktree, vec![String::from(".rit")]);
        if gitdir.exists() {
            return Err(String::from("Worktree .git directory already exists"));
        }
    }
    pub fn from_existing_folder(worktree: PathBuf) -> Result<Self, String> {
        if !worktree.exists() {
            return Err(format!(
                "Not able to access current directory: {:?}",
                worktree
            ));
        }

        let mut gitdir = worktree.clone();
        gitdir.push(".rit");
        if !gitdir.exists() {
            return Err(String::from("Worktree .git directory does not exists"));
        }

        let mut gitconfig = gitdir.clone();
        gitconfig.push("config");
        let mut gitconfig_ini = Ini::new();
        gitconfig_ini
            .load(gitconfig)
            .map_err(|e| format!("Error loading .git directory config: {}", e))?;
        let repo_config = Config::from_ini(gitconfig_ini);

        Ok(Self {
            worktree,
            gitdir,
            config: repo_config,
        })
    }

    // fn create_dir(base_path: &PathBuf, paths: Vec<String>) -> Result<(), String> {
    //     let mut new_path = base_path.clone();
    //     for path in paths {
    //         new_path.push(path);
    //     }
    //     if let Err(e) = create_dir(&new_path) {
    //         Err(format!("Not able to create dir, {:?}: {}", new_path, e))
    //     } else {
    //         Ok(())
    //     }
    // }
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
