use std::{
    fs::{remove_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};

use configparser::ini::Ini;

use crate::{create_dir, create_path, Config};

/// When initializing the struct, you should use [Self::init_worktree] or [Self::find_worktree_root]
pub struct Repository {
    pub worktree: PathBuf,
    pub gitdir: PathBuf,
    pub config: Config,
}
impl Repository {
    pub fn clean_worktree(worktree: PathBuf, git_dir_path: &str) -> Result<(), String> {
        let gitdir = create_path(&worktree, vec![git_dir_path.to_owned()]);
        remove_dir_all(gitdir).map_err(|e| format!("Unable to remove worktree, {}", e))?;
        Ok(())
    }
    pub fn init_worktree(worktree: PathBuf, git_dir_path: &str) -> Result<Self, String> {
        let gitdir = create_path(&worktree, vec![git_dir_path.to_owned()]);
        create_dir(&gitdir)?;
        create_dir(&create_path(&gitdir, vec![String::from("objects")]))?;
        let refsdir = create_path(&gitdir, vec![String::from("refs")]);
        create_dir(&refsdir)?;
        create_dir(&create_path(&refsdir, vec![String::from("heads")]))?;
        create_dir(&create_path(&refsdir, vec![String::from("tags")]))?;
        Self::create_head(&gitdir)?;
        Self::create_config(&gitdir)?;
        Self::create_description(&gitdir)?;

        Self::from_worktree_root(worktree, git_dir_path)
    }
    fn create_head(gitdir: &Path) -> Result<(), String> {
        let mut path = gitdir.to_path_buf();
        path.push("HEAD");
        File::create(path.clone())
            .map_err(|e| format!("Error creating empty file, {:?}: {}", path, e))?;
        Ok(())
    }
    fn create_config(gitdir: &Path) -> Result<(), String> {
        let mut path = gitdir.to_path_buf();
        path.push("config");
        let default_config = Config::default();
        let default_ini = default_config.to_ini();
        default_ini
            .write(path)
            .map_err(|e| format!("Error writing git config file: {}", e))?;
        Ok(())
    }
    fn create_description(gitdir: &Path) -> Result<(), String> {
        let mut path = gitdir.to_path_buf();
        path.push("description");
        let mut description = File::create(path.clone())
            .map_err(|e| format!("Error creating empty file, {:?}: {}", path, e))?;
        description
            .write_all(b"Unnamed repository; edit this file 'description' to name the repository.")
            .map_err(|e| format!("Error writing to git description: {}", e))?;
        Ok(())
    }
    pub fn from_worktree_root(worktree_root: PathBuf, git_dir_path: &str) -> Result<Self, String> {
        let gitdir = create_path(&worktree_root, vec![git_dir_path.to_owned()]);

        let mut gitconfig = gitdir.clone();
        gitconfig.push("config");
        let mut gitconfig_ini = Ini::new();
        gitconfig_ini
            .load(gitconfig)
            .map_err(|e| format!("Error loading .git directory config: {}", e))?;
        let repo_config = Config::from_ini(gitconfig_ini);

        Ok(Self {
            worktree: worktree_root,
            gitdir,
            config: repo_config,
        })
    }
    pub fn find_worktree_root(current_dir: PathBuf, git_dir_path: &str) -> Option<Self> {
        // TODO: Check if errors on the cannonicalize needs to be dealt
        let mut current_dir = current_dir.canonicalize().unwrap();
        while current_dir != PathBuf::from("/") {
            let potential_worktree_root = create_path(&current_dir, vec![git_dir_path.to_owned()]);
            if potential_worktree_root.exists() {
                return Some(Self::from_worktree_root(current_dir, git_dir_path).unwrap());
            }
            current_dir.push("../");
            current_dir = current_dir.canonicalize().unwrap();
        }
        None
    }
}
