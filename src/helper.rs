use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use hex::encode;
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
pub enum Object {
    Commit,
    Tree,
    Tag,
    Blob,
}
#[allow(clippy::inherent_to_string)]
impl Object {
    pub fn read_from_sha(repo: Repository, hash: String) -> Result<Option<()>, String> {
        // TODO: hash should be computed by the object itself
        // Using SHA-1 for now
        // There have been talks to shift to SHA-2
        // TLDR: Git has implemented the necessary software, but still have not transitioned yet
        let hash: Vec<char> = hash.chars().collect();
        let directory: String = hash[..2].iter().collect();
        let filename: String = hash[2..].iter().collect();
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
        let mut header: Vec<u8> = Vec::new();
        let header_delimiter = 0x20u8;
        let mut length: Vec<u8> = Vec::new();
        let length_delimiter = 0x0u8;
        let mut raw_file_content_iter = raw_file_contents.into_iter().peekable();
        while raw_file_content_iter.peek().is_some() {
            let b = raw_file_content_iter.next().unwrap();
            if b == header_delimiter {
                break;
            }
            header.push(b);
        }
        while raw_file_content_iter.peek().is_some() {
            let b = raw_file_content_iter.next().unwrap();
            if b == length_delimiter {
                break;
            }
            length.push(b);
        }
        let content: Vec<u8> = raw_file_content_iter.collect();

        let header = String::from_utf8(header).unwrap();
        let length = String::from_utf8(length).unwrap();
        let content = String::from_utf8(content).unwrap();

        Ok(Some(()))
    }
    pub fn write_to_repo(self, repo: Repository, data: Vec<u8>) -> Result<String, String> {
        // TODO: Sha should not be return, should be owned by the object
        // TODO: data should also already be owned by the object
        let header = self.to_string();
        let content_length = data.len().to_string();
        let final_content = [
            header.as_bytes(),
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
        // Should be [u8; 20]
        let hash: [u8; 20] = hasher
            .finalize()
            .as_slice()
            .try_into()
            .expect("Wrong length");

        let directory = encode(&hash[..1]);
        let filename = encode(&hash[1..]);
        let object_directory = create_path(
            &repo.gitdir,
            vec![String::from("objects"), directory.clone()],
        );
        let object_file_path = create_path(
            &repo.gitdir,
            vec![String::from("objects"), directory, filename],
        );
        // Create directory if not already exists
        match fs::create_dir(object_directory.clone()) {
            Ok(_) => Ok(()),
            Err(e) => match e.kind() {
                io::ErrorKind::AlreadyExists => Ok(()),
                _ => Err(format!(
                    "Error creating object directory, {:?}, {}",
                    object_directory, e
                )),
            },
        }?;
        let mut object_file = File::create(object_file_path.clone())
            .map_err(|e| format!("Error creating object file, {:?}: {}", object_file_path, e))?;
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&final_content)
            .map_err(|_| "Error encoding data")?;
        let compressed_content = encoder
            .finish()
            .map_err(|e| format!("Error writing encoded data: {}", e))?;
        object_file
            .write_all(&compressed_content)
            .map_err(|e| format!("Error writing encoded data to file: {}", e))?;
        Ok(encode(hash))
    }
    fn to_string(&self) -> String {
        match self {
            Self::Commit => String::from("commit"),
            Self::Blob => String::from("blob"),
            Self::Tag => String::from("tag"),
            Self::Tree => String::from("tree"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hex() {
        let sha: [u8; 20] = [
            0b11100110, 0b01110011, 0b11010001, 0b10110111, 0b11101010, 0b10100000, 0b10101010,
            0b00000001, 0b10110101, 0b10111100, 0b00100100, 0b01000010, 0b11010101, 0b01110000,
            0b10100111, 0b01100101, 0b10111101, 0b10101010, 0b11100111, 0b01010001,
        ];
        let directory = encode(&sha[0..1]);
        let filename = encode(&sha[1..]);
        assert_eq!(directory, String::from("e6"));
        assert_eq!(
            filename,
            String::from("73d1b7eaa0aa01b5bc2442d570a765bdaae751")
        );
    }
}
