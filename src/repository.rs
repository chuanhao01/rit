use std::{fs::create_dir, path::PathBuf};

struct Repository {
    worktree: PathBuf,
    gitdir: PathBuf,
}
impl Repository {
    pub fn init_worktree(worktree: PathBuf) -> Result<Self, String> {
        if !worktree.exists() {
            return Err(format!(
                "Not able to access current directory: {:?}",
                worktree
            ));
        }

        let mut gitdir = worktree.clone();
        gitdir.push(".rit");
        if gitdir.exists() {
            return Err(String::from("Worktree .git directory already exists"));
        }
        Self::create_dir(&gitdir, Vec::new())?;

        Ok(Self { worktree, gitdir })
    }
    // pub fn from_existing_folder(worktree: PathBuf) -> Result<Self, String> {}
    fn create_dir(base_path: &PathBuf, paths: Vec<String>) -> Result<(), String> {
        let mut new_path = base_path.clone();
        for path in paths {
            new_path.push(path);
        }
        if let Err(e) = create_dir(new_path.clone()) {
            Err(format!("Not able to create dir, {:?}: {}", new_path, e))
        } else {
            Ok(())
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
