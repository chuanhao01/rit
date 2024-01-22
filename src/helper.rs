use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};

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

// TODO: Choice of picking between hashing algos
// Because git itself is trying to migrate over to SHA-256 (SHA2)
pub enum Object {}
impl Object {
    pub fn read_from_sha(repo: Repository, sha: [u8; 40]) -> Result<Option<()>, String> {
        // TODO: sha should be computed by the object itself
        let directory = String::from_utf8(sha[..2].to_owned())
            .map_err(|e| format!("Invalid sha, {:?}: {}", sha, e))?;
        let filename = String::from_utf8(sha[2..].to_owned())
            .map_err(|e| format!("Invalid sha, {:?}: {}", sha, e))?;
        let object_file_path = create_path(
            &repo.gitdir,
            vec![String::from("objects"), directory, filename],
        );
        let object_file = File::open(object_file_path.clone())
            .map_err(|e| format!("Error opening file, {:?}: {}", object_file_path, e))?;
        let mut raw_file_contents: Vec<u8> = Vec::new();
        ZlibDecoder::new(object_file)
            .read_to_end(&mut raw_file_contents)
            .map_err(|_| "Error zlib decode file contents")?;
        println!("{:?}", raw_file_contents);
        Ok(Some(()))
    }
    pub fn write_to_repo(repo: Repository, data: Vec<u8>, format: String) -> Result<(), String> {
        // TODO: Sha should not be return, should be owned by the object
        // TODO: data should also already be owned by the object
        let header = format.as_bytes();
        let content_length = data.len().to_string();
        let final_content = [
            header,
            b"\x20",
            content_length.as_bytes(),
            b"\x00",
            data.as_slice(),
        ]
        .concat();
        // DANGER
        // This is due to rust-analyzer not being able to get the correct types
        // Also the generic array does not have the correct length
        // Related GH issue: https://github.com/RustCrypto/hashes/issues/441, https://github.com/rust-lang/rust-analyzer/issues/15242
        // sha1 crate: https://docs.rs/sha1/latest/sha1/
        // SO Ans: https://stackoverflow.com/questions/59376378/how-can-i-turn-a-genericarrayt-into-an-array-of-the-same-length
        let mut hasher = Sha1::new();
        hasher.update(&final_content);
        let hash = hasher.finalize();
        dbg!(hash);

        // let directory = String::from_utf8(hash[..2].to_owned())
        //     .map_err(|e| format!("Invalid sha, {:?}: {}", hash, e))?;
        // let filename = String::from_utf8(hash[2..].to_owned())
        //     .map_err(|e| format!("Invalid sha, {:?}: {}", hash, e))?;
        // let object_file_path = create_path(
        //     &repo.gitdir,
        //     vec![String::from("objects"), directory, filename],
        // );
        // let mut object_file = File::create(object_file_path.clone())
        //     .map_err(|e| format!("Error creating object file, {:?}: {}", object_file_path, e))?;
        // let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        // encoder
        //     .write_all(&final_content)
        //     .map_err(|_| "Error encoding data")?;
        // let compressed_content = encoder
        //     .finish()
        //     .map_err(|e| format!("Error writing encoded data: {}", e))?;
        // object_file
        //     .write_all(&compressed_content)
        //     .map_err(|e| format!("Error writing encoded data to file: {}", e))?;
        Ok(())
    }
}
