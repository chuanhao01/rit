use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use configparser::ini::Ini;

use crate::{create_dir, create_path, Config};

pub struct Repository {
    worktree: PathBuf,
    gitdir: PathBuf,
    config: Config,
}
impl Repository {
    pub fn init_worktree(worktree: PathBuf) -> Result<Self, String> {
        let gitdir = create_path(&worktree, vec![String::from(".rit")]);
        create_dir(&gitdir)?;
        create_dir(&create_path(&gitdir, vec![String::from("objects")]))?;
        let refsdir = create_path(&gitdir, vec![String::from("refs")]);
        create_dir(&refsdir)?;
        create_dir(&create_path(&refsdir, vec![String::from("heads")]))?;
        create_dir(&create_path(&refsdir, vec![String::from("tags")]))?;
        Self::create_head(&gitdir)?;
        Self::create_config(&gitdir)?;
        Self::create_description(&gitdir)?;

        Self::from_existing_folder(worktree)
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
    pub fn from_existing_folder(worktree: PathBuf) -> Result<Self, String> {
        let gitdir = create_path(&worktree, vec![String::from(".rit")]);

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
}
