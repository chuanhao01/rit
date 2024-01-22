use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::Repository;

pub fn create_path(base_path: &Path, paths: Vec<String>) -> PathBuf {
    let mut base_path = base_path.to_path_buf();
    for path in paths {
        base_path.push(path);
    }
    base_path
}

pub fn create_dir(path: &Path) -> Result<(), String> {
    fs::create_dir(path).map_err(|e| format!("Error creating dir, {:?}: {}", path, e))
}

enum Object {}
impl Object {
    pub fn read_from_sha(repo: Repository, sha: [u8; 40]) -> Result<Option<Self>, String> {
        let directory = String::from_utf8(sha[..2].to_owned())
            .map_err(|e| format!("Invalid sha, {:?}: {}", sha, e))?;
        let filename = String::from_utf8(sha[2..].to_owned())
            .map_err(|e| format!("Invalid sha, {:?}: {}", sha, e))?;
        let object_file = create_path(
            &repo.gitdir,
            vec![String::from("objects"), directory, filename],
        );
    }
}
