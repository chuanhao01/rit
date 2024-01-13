use std::path::PathBuf;

pub fn create_path(base_path: &PathBuf, paths: [&str]) -> PathBuf{
    let mut base_path = base_path.clone();
    for path in paths{
        base_path.push(path);
    }
    base_path
}
