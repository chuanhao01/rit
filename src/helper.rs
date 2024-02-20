use std::{
    fs,
    path::{Path, PathBuf},
};

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

pub fn hex_to_hex_byte(s: &str) -> Result<Vec<u8>, String> {
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| format!("Failed to decode hexstr {}, {}", s, e))
        })
        .collect()
}
